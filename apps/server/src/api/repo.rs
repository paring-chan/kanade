use std::sync::Arc;

use api_types::{
    ErrorResponse, GetRepoResponse, PipelineListResponse, PipelineResponse,
    RepoCreateEndpointResponse, RepoCreateRequest, RepoCreateResponse, RepoResponse, TeamResponse,
    UserResponse,
};
use chrono::{DateTime, Utc};
use garde::Validate;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use poem::web::Data;
use poem_openapi::{
    OpenApi,
    param::{Path, Query},
    payload::Json,
};
use rand::rand_core::UnwrapErr;
use sqlx::{PgPool, prelude::FromRow};
use ssh_key::PrivateKey;
use uuid::Uuid;

use crate::{
    api::security::ApiKeyAuth,
    crypto::CryptoEngine,
    data::db::{EventType, PipelineStatus},
    error::AppError,
    forges::AllForges,
};

pub struct RepoApi;

#[OpenApi(prefix_path = "/repos", tag = "super::ApiTags::Repo")]
impl RepoApi {
    #[oai(path = "/", method = "post")]
    async fn create(
        &self,
        Data(db): Data<&PgPool>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Json(payload): Json<RepoCreateRequest>,
        Data(forges): Data<&Arc<AllForges>>,
        Data(crypto): Data<&Arc<CryptoEngine>>,
    ) -> poem::Result<RepoCreateEndpointResponse> {
        self._create(db, user_id, payload, forges, crypto)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip(self, db, forges, crypto, payload), err(Debug))]
    async fn _create(
        &self,
        db: &PgPool,
        user_id: Uuid,
        payload: RepoCreateRequest,
        forges: &AllForges,
        crypto: &Arc<CryptoEngine>,
    ) -> crate::Result<RepoCreateEndpointResponse> {
        match payload.validate() {
            Ok(_) => {}
            Err(report) => {
                return Ok(RepoCreateEndpointResponse::ValidationFailed(Json(
                    serde_json::to_value(report)?,
                )));
            }
        }

        let auth = forges
            .get_forge_auth(user_id, payload.forge_id)
            .await?
            .ok_or(AppError::ForgeNotLinked)?;

        let upstream_repo = forges
            .get_repo(&auth, &payload.forge_repo_id)
            .await?
            .ok_or(AppError::UpstreamRepoNotFound)?;

        let exists: (bool, bool) = sqlx::query_as::<_, (bool, bool)>(
            r#"
            SELECT
                EXISTS(SELECT 1 FROM repo WHERE team_id = $1 AND slug = $2),
                EXISTS(SELECT 1 FROM repo WHERE forge_id = $3 AND forge_repo_id = $4)
            "#,
        )
        .bind(&payload.team_id)
        .bind(&payload.slug)
        .bind(&payload.forge_id)
        .bind(&upstream_repo.id)
        .fetch_one(db)
        .await?;

        if exists.0 {
            return Ok(RepoCreateEndpointResponse::Conflict(Json(ErrorResponse {
                message: "해당 팀 내에 이미 동일한 슬러그를 가진 프로젝트가 존재합니다.".into(),
            })));
        }
        if exists.1 {
            return Ok(RepoCreateEndpointResponse::Conflict(Json(ErrorResponse {
                message: "이 저장소는 이미 다른 프로젝트에 연동되어 있습니다.".into(),
            })));
        }

        let repo_id = Uuid::new_v4();

        let mut webhook_token = [0; 32];
        OsRng.fill_bytes(&mut webhook_token);
        let webhook_token = hex::encode(webhook_token);

        let private_key = PrivateKey::random(
            &mut UnwrapErr(rand::rngs::SysRng),
            ssh_key::Algorithm::Ed25519,
        )
        .map_err(|e| AppError::InternalError(e.into()))?;

        let private_key_encoded = private_key
            .to_openssh(ssh_key::LineEnding::LF)
            .map_err(|e| AppError::InternalError(e.into()))?;

        forges
            .add_ssh_key(&auth, &upstream_repo, private_key.public_key())
            .await?;

        forges
            .setup_webhook(&auth, &upstream_repo, &webhook_token, repo_id)
            .await?;

        let encrypted_webhook_token = crypto.encrypt(&webhook_token)?;
        let encrypted_ssh_key = crypto.encrypt(&private_key_encoded)?;
        let mut tx = db.begin().await?;

        #[derive(FromRow)]
        struct InsertedRow {
            id: Uuid,
            repo_slug: String,
            team_slug: String,
        }

        let res = sqlx::query_as::<_, InsertedRow>(
            r#"
            WITH inserted AS (
                INSERT INTO repo
                    (id, name, slug, team_id, forge_id, forge_repo_id, forge_webhook_token, ssh_key, repo_url, created_by)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
            )
            SELECT
                r.id, r.slug as repo_slug, t.slug as team_slug
            FROM inserted r
            INNER JOIN team t ON t.id = r.team_id
            "#,
        )
        .bind(repo_id)
        .bind(&payload.name)
        .bind(&payload.slug)
        .bind(&payload.team_id)
        .bind(&payload.forge_id)
        .bind(&upstream_repo.id)
        .bind(&encrypted_webhook_token)
        .bind(&encrypted_ssh_key)
        .bind(&upstream_repo.url)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(RepoCreateEndpointResponse::Ok(Json(RepoCreateResponse {
            id: res.id,
            repo_slug: res.repo_slug,
            team_slug: res.team_slug,
        })))
    }

    #[oai(path = "/:team/:repo", method = "get")]
    async fn get_by_slug(
        &self,
        Data(db): Data<&PgPool>,
        Path(team): Path<String>,
        Path(repo): Path<String>,
        ApiKeyAuth(user_id): ApiKeyAuth,
    ) -> poem::Result<GetRepoResponse> {
        self._get_by_slug(db, user_id, &team, &repo)
            .await
            .map_err(Into::into)
    }

    async fn _get_by_slug(
        &self,
        db: &PgPool,
        user_id: Uuid,
        team: &str,
        repo: &str,
    ) -> crate::Result<GetRepoResponse> {
        #[derive(FromRow)]
        struct RepoRow {
            r_id: Uuid,
            r_name: String,
            r_slug: String,
            r_repo_url: String,
            r_created_at: DateTime<Utc>,
            r_updated_at: DateTime<Utc>,

            t_id: Uuid,
            t_name: String,
            t_slug: String,

            t_created_at: DateTime<Utc>,
            t_updated_at: DateTime<Utc>,
        }

        let result = sqlx::query_as::<_, RepoRow>(
            r#"
            SELECT
                r.id as r_id,
                r.name as r_name,
                r.slug as r_slug,
                r.repo_url as r_repo_url,
                r.created_at as r_created_at,
                r.updated_at as r_updated_at,

                t.id as t_id,
                t.name as t_name,
                t.slug as t_slug,
                t.created_at as t_created_at,
                t.updated_at as t_updated_at
            FROM repo r
            INNER JOIN team t ON t.id = r.team_id
            INNER JOIN user_team ut ON ut.team_id = t.id
            WHERE
                ut.user_id = $1 AND
                t.slug = $2 AND
                r.slug = $3
            "#,
        )
        .bind(user_id)
        .bind(team)
        .bind(repo)
        .fetch_optional(db)
        .await?;

        match result {
            Some(row) => Ok(GetRepoResponse::Ok(Json(RepoResponse {
                id: row.r_id,
                name: row.r_name,
                slug: row.r_slug,
                upstream_url: row.r_repo_url,
                created_at: row.r_created_at,
                updated_at: row.r_updated_at,
                team: TeamResponse {
                    id: row.t_id,
                    name: row.t_name,
                    slug: row.t_slug,
                    created_at: row.t_created_at,
                    updated_at: row.t_updated_at,
                },
            }))),
            None => Ok(GetRepoResponse::NotFound(Json(ErrorResponse {
                message: "repo not found".to_string(),
            }))),
        }
    }

    #[oai(path = "/:team/:repo/pipelines", method = "get")]
    async fn list_pipelines(
        &self,
        Data(db): Data<&PgPool>,
        Path(team): Path<String>,
        Path(repo): Path<String>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Query(cursor): Query<Option<Uuid>>,
    ) -> poem::Result<Json<PipelineListResponse>> {
        self._list_pipelines(db, user_id, &team, &repo, cursor)
            .await
            .map(Json)
            .map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _list_pipelines(
        &self,
        db: &PgPool,
        user_id: Uuid,
        team: &str,
        repo: &str,
        cursor: Option<Uuid>,
    ) -> crate::Result<PipelineListResponse> {
        #[derive(FromRow)]
        struct PipelineQueryRow {
            p_id: Uuid,
            p_serial: i32,
            p_repo_id: Uuid,
            p_title: Option<String>,
            p_triggered_by: String,
            p_event_type: EventType,
            p_event_payload: sqlx::types::Json<serde_json::Value>,
            p_git_ref: Option<String>,
            p_git_commit_id: String,
            p_status: PipelineStatus,
            p_created_at: DateTime<Utc>,
            p_updated_at: DateTime<Utc>,

            tu_id: Option<Uuid>,
            tu_username: Option<String>,
            tu_nick: Option<String>,
            tu_email: Option<String>,
            tu_avatar_url: Option<String>,
            tu_created_at: Option<DateTime<Utc>>,
            tu_updated_at: Option<DateTime<Utc>>,
        }

        let result = sqlx::query_as::<_, PipelineQueryRow>(
            r#"
            SELECT
                p.id as p_id,
                p.serial as p_serial,
                p.repo_id p_repo_id,
                p.title p_title,
                p.triggered_by p_triggered_by,
                p.event_type as p_event_type,
                p.event_payload as p_event_payload,
                p.git_ref as p_git_ref,
                p.git_commit_id as p_git_commit_id,
                p.status as p_status,
                p.created_at as p_created_at,
                p.updated_at as p_updated_at,

                tu.id as tu_id,
                tu.username as tu_username,
                tu.nick as tu_nick,
                tu.email as tu_email,
                tu.avatar_url as tu_avatar_url,
                tu.created_at as tu_created_at,
                tu.updated_at as tu_updated_at
            FROM repo r
            INNER JOIN team t ON t.id = r.team_id
            INNER JOIN user_team ut ON ut.team_id = t.id
            INNER JOIN pipeline p ON p.repo_id = r.id
            LEFT JOIN "user" tu ON p.triggered_by_user = tu.id
            WHERE
                ut.user_id = $1 AND
                t.slug = $2 AND
                r.slug = $3 AND
                ($4 IS NULL OR p.id < $4)
            ORDER BY p.id DESC
            LIMIT 20
            "#,
        )
        .bind(user_id)
        .bind(team)
        .bind(repo)
        .bind(cursor)
        .fetch_all(db)
        .await?;

        if result.is_empty() {
            return Ok(PipelineListResponse {
                items: vec![],
                next_cursor: None,
            });
        }

        let len = result.len();
        let last = result[len - 1].p_id;

        Ok(PipelineListResponse {
            items: result
                .into_iter()
                .map(|x| PipelineResponse {
                    id: x.p_id,
                    serial: x.p_serial,
                    repo_id: x.p_repo_id,
                    title: x.p_title,
                    triggered_by: x.p_triggered_by,
                    triggered_by_user: x.tu_id.map(|id| UserResponse {
                        id,
                        username: x.tu_username.expect("must exist"),
                        nick: x.tu_nick,
                        email: x.tu_email,
                        avatar_url: x.tu_avatar_url,
                        created_at: x.tu_created_at.expect("must exist"),
                        updated_at: x.tu_updated_at.expect("must exist"),
                    }),
                    event_type: x.p_event_type.into(),
                    event_payload: x.p_event_payload.0,
                    git_ref: x.p_git_ref,
                    git_commit_id: x.p_git_commit_id,
                    status: x.p_status.into(),
                    created_at: x.p_created_at,
                    updated_at: x.p_updated_at,
                })
                .collect(),
            next_cursor: Some(last).filter(|_| len == 20),
        })
    }
}
