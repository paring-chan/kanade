use std::{collections::HashMap, sync::Arc};

use event_evaluator::{script::EvalContext, types::EnvDefinition};
use hmac::{KeyInit, Mac};
use poem::{
    Body, EndpointExt, Request, Route,
    endpoint::BoxEndpoint,
    error::BadRequest,
    handler,
    http::{StatusCode, header::CONTENT_TYPE},
    post,
    web::{Data, Query},
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::{
    data::db::{EventType, JobStatus, PipelineStatus},
    error::AppError,
    forges::AllForges,
};

pub fn routes() -> BoxEndpoint<'static> {
    Route::new().just_at(post(forgejo_webhook)).boxed()
}

#[derive(Deserialize)]
struct WebhookQueryParams {
    repo: Uuid,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct WebhookMessage {
    pub r#ref: String,
    pub after: String,
    pub repository: ForgejoRepository,
    pub pusher: ForgejoUser,
    pub sender: ForgejoUser,
    pub head_commit: ForgejoCommit,
}

#[derive(Debug, Deserialize, Clone)]
struct ForgejoCommit {
    pub message: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ForgejoRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ForgejoUser {
    pub id: u64,
    pub login: String,
    pub full_name: String,
    pub email: String,
    pub avatar_url: String,
    pub username: String,
}

#[handler]
async fn forgejo_webhook(
    req: &Request,
    body: Body,
    Data(db): Data<&PgPool>,
    crypto: Data<&Arc<crate::crypto::CryptoEngine>>,
    Query(query): Query<WebhookQueryParams>,
    Data(forges): Data<&Arc<AllForges>>,
) -> poem::Result<()> {
    if req.header(CONTENT_TYPE) != Some("application/json") {
        return Err(poem::Error::from_string(
            "invalid content type",
            StatusCode::BAD_REQUEST,
        ));
    }

    let sig = req
        .header("x-forgejo-signature")
        .ok_or_else(|| poem::Error::from_string("missing signature", StatusCode::BAD_REQUEST))?;
    let event = req
        .header("x-forgejo-event")
        .ok_or_else(|| poem::Error::from_string("missing event", StatusCode::BAD_REQUEST))?;

    let body_str = body.into_string().await?;
    let body = serde_json::from_str::<WebhookMessage>(&body_str).map_err(BadRequest)?;

    let repo_row = sqlx::query!(
        r#"SELECT forge_id, forge_repo_id, created_by, forge_webhook_token FROM repo WHERE id = $1"#,
        query.repo
    )
    .fetch_one(db)
    .await
    .map_err(|_| poem::Error::from_string("repository not found", StatusCode::NOT_FOUND))?;

    let webhook_token = crypto.decrypt(&repo_row.forge_webhook_token).map_err(|e| {
        error!("failed to decrypt webhook token: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(webhook_token.expose_secret().as_bytes())
        .map_err(|e| {
            debug!("invalid signature: {e}");
            poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    mac.update(body_str.as_bytes());

    let mut expected = [0u8; 32];
    hex::decode_to_slice(&sig, &mut expected).map_err(BadRequest)?;

    if mac.verify_slice(&expected).is_err() {
        return Err(poem::Error::from_string(
            "signature verification failed",
            StatusCode::BAD_REQUEST,
        ));
    }

    let event = match event {
        "push" => EventType::Push,
        unhandled => {
            debug!("unhandled event: {unhandled}");
            return Ok(());
        }
    };

    let auth = forges
        .get_forge_auth(repo_row.created_by, repo_row.forge_id)
        .await?
        .ok_or(AppError::ForgeNotLinked)?;

    debug!("auth: {auth:?}");

    let upstream_repo = forges
        .get_repo(&auth, &repo_row.forge_repo_id)
        .await?
        .ok_or_else(|| poem::Error::from_string("repo not found", StatusCode::NOT_FOUND))?;

    debug!("repo: {upstream_repo:?}");

    let script = match forges
        .get_repo_script(&auth, &upstream_repo, &body.after)
        .await
        .inspect_err(|e| {
            error!("failed to fetch repo config file: {e:?}");
        })? {
        Some(v) => v,
        None => {
            debug!("config file not found");
            return Ok(());
        }
    };

    let pipelines = tokio::task::spawn_blocking({
        let body = body.clone();
        let event = match &event {
            EventType::Push => "push",
            EventType::Tag => "tag",
            EventType::Release => "release",
            EventType::PullRequest => "pull_request",
            EventType::Cron => "cron",
            EventType::Manual => "manual",
        };
        move || {
            event_evaluator::evaluate(
                &script,
                EvalContext {
                    event,
                    branch: body
                        .r#ref
                        .strip_prefix("refs/heads/")
                        .map(|x| x.to_string()),
                    tag: body.r#ref.strip_prefix("refs/tags/").map(|x| x.to_string()),
                    r#ref: body.r#ref.clone(),
                    args: Default::default(),
                    default_image: "oven/bun:latest".to_string(),
                    default_shell: "/bin/sh".to_string(),
                    pipelines: Default::default(),
                },
            )
            .map_err(|e| {
                return poem::Error::from_string(
                    format!("failed to evaluate script: {e}"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            })
        }
    })
    .await
    .map_err(AppError::from)??;

    let mut tx = db.begin().await.map_err(|e| {
        error!("failed to start tx: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    sqlx::query!("SELECT FROM repo WHERE id = $1 FOR UPDATE", query.repo)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("failed to lock repo: {e}");
            poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    for pipeline in pipelines {
        sqlx::query!(
            r#"
            INSERT INTO pipeline
                (
                    id, repo_id, title, triggered_by, triggered_by_user,
                    event_type, event_payload, git_ref, git_commit_id, status,
                    serial
                )
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    (SELECT COALESCE(MAX(serial), 0) + 1 FROM pipeline WHERE repo_id = $2)
                )
            "#,
            pipeline.id,
            query.repo,
            format!("{} ({})", &body.head_commit.message, pipeline.name),
            body.sender.username,
            Option::<Uuid>::None as Option<Uuid>,
            event.clone() as EventType,
            Json(json!({})) as Json<serde_json::Value>,
            body.r#ref,
            body.after,
            PipelineStatus::Queued as PipelineStatus
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("failed to insert pipeline: {e}");
            poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        for mut job in pipeline.jobs {
            job.env.insert(
                "CLONE_URL".to_string(),
                EnvDefinition::Static(upstream_repo.ssh_url.clone()),
            );

            sqlx::query!(
                r#"
                    INSERT INTO pipeline_job
                        (id, pipeline_id, key, name, image, timeout, status, env)
                    VALUES
                        ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                job.id,
                pipeline.id,
                job.key,
                job.name,
                job.image,
                job.timeout.min(60 * 6), // 최대 6시간
                JobStatus::Pending as JobStatus,
                Json(job.env) as Json<HashMap<String, EnvDefinition>>,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("failed to insert job: {e}");
                poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            for parent_id in job.parents {
                sqlx::query!(
                    r#"
                    INSERT INTO pipeline_job_depend
                        (upstream_id, downstream_id)
                    VALUES
                        ($1, $2)
                    "#,
                    parent_id,
                    job.id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("failed to insert job depend: {e}");
                    poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
                })?;
            }

            for (idx, step) in job.steps.into_iter().enumerate() {
                sqlx::query!(
                    r#"
                    INSERT INTO pipeline_job_step
                        (id, job_id, name, ordering, command, status, env)
                    VALUES
                        ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    step.id,
                    job.id,
                    step.name,
                    (idx + 1) as i32,
                    step.command,
                    JobStatus::Pending as JobStatus,
                    Json(step.env) as Json<HashMap<String, EnvDefinition>>,
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("failed to insert job step: {e}");
                    poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
                })?;
            }
        }
    }

    tx.commit().await.map_err(|e| {
        error!("failed to commit transaction: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(())
}
