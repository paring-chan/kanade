use std::sync::Arc;

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
use sqlx::prelude::FromRow;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::data::db::{EventType, JobStatus, PipelineStatus};

pub fn routes() -> BoxEndpoint<'static> {
    Route::new().just_at(post(forgejo_webhook)).boxed()
}

#[derive(Deserialize)]
struct WebhookQueryParams {
    repo: Uuid,
}

#[derive(Debug, FromRow)]
struct RepoWebhookToken {
    forge_webhook_token: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct WebhookMessage {
    pub r#ref: String,
    pub after: String,
    pub repository: ForgejoRepository,
    pub pusher: ForgejoUser,
    pub sender: ForgejoUser,
}

#[derive(Debug, Deserialize)]
struct ForgejoRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
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

    let repo_row =
        sqlx::query_as::<_, RepoWebhookToken>("SELECT forge_webhook_token FROM repo WHERE id = $1")
            .bind(query.repo)
            .fetch_one(db)
            .await
            .map_err(|_| poem::Error::from_string("repository not found", StatusCode::NOT_FOUND))?;

    let webhook_token = crypto.decrypt(&repo_row.forge_webhook_token).map_err(|e| {
        error!("failed to decrypt webhook token: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(webhook_token.expose_secret().as_bytes())
        .map_err(|_| {
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
        _ => return Ok(()),
    };

    let mut tx = db.begin().await.map_err(|e| {
        error!("failed to start tx: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let pipeline_id = Uuid::now_v7();

    sqlx::query(
        r#"
        SELECT FROM repo WHERE id = $1 FOR UPDATE;
    "#,
    )
    .bind(query.repo)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("failed to lock repository: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    sqlx::query(
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
    )
    .bind(pipeline_id)
    .bind(query.repo)
    .bind("Test")
    .bind("wow")
    .bind(Option::<Uuid>::None)
    .bind(event)
    .bind(Json(json!({})))
    .bind(&body.r#ref)
    .bind(&body.after)
    .bind(PipelineStatus::Evaluating)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("failed to insert pipeline: {e}");
        return poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR);
    })?;

    let evaluation_id = Uuid::now_v7();

    sqlx::query(
        r#"
            INSERT INTO pipeline_job
                (id, pipeline_id, key, name, image, timeout)
            VALUES
                ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(evaluation_id)
    .bind(pipeline_id)
    .bind("evaluate")
    .bind("Evaluate Pipeline")
    .bind("oven/bun:latest")
    // .bind("oci.pari.ng/kanade/bun-evaluator")
    .bind(5)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("failed to insert job: {e}");
        return poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR);
    })?;

    let step_id = Uuid::now_v7();

    sqlx::query(
        r#"
            INSERT INTO pipeline_job_step
                (id, job_id, name, ordering, command)
            VALUES
                ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(step_id)
    .bind(evaluation_id)
    .bind("Evaluate Pipeline Jobs")
    .bind(1)
    .bind("bun test.ts")
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("failed to insert job step: {e}");
        return poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR);
    })?;

    let run_id = Uuid::now_v7();

    sqlx::query(
        r#"
            INSERT INTO pipeline_job_run
                (id, job_id, attempt_serial, status)
            VALUES
                ($1, $2, $3, $4)
        "#,
    )
    .bind(run_id)
    .bind(evaluation_id)
    .bind(1)
    .bind(JobStatus::Pending)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("failed to insert job run: {e}");
        return poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR);
    })?;

    let step_run_id = Uuid::now_v7();

    sqlx::query(
        r#"
            INSERT INTO pipeline_job_step_run
                (id, run_id, step_id, status)
            VALUES
                ($1, $2, $3, $4)
        "#,
    )
    .bind(step_run_id)
    .bind(run_id)
    .bind(step_id)
    .bind(JobStatus::Pending)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("failed to insert job step run: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    tx.commit().await.map_err(|e| {
        error!("failed to commit webhook transaction: {e}");
        poem::Error::from_string("internal error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(())
}
