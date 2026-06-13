use std::sync::Arc;

use api_types::{
    ErrorResponse, GetRepoResponse, RepoCreateEndpointResponse, RepoCreateRequest,
    RepoCreateResponse, RepoResponse, TeamResponse,
};
use chrono::{DateTime, Utc};
use garde::Validate;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use poem::web::Data;
use poem_openapi::{NewType, Object, OpenApi, param::Path, payload::Json};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{api::security::ApiKeyAuth, crypto::CryptoEngine, error::AppError, forges::AllForges};

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

        forges
            .setup_webhook(&auth, &upstream_repo, &webhook_token, repo_id)
            .await?;

        let encrypted_webhook_token = crypto.encrypt(&webhook_token)?;
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
                    (id, name, slug, team_id, forge_id, forge_repo_id, forge_webhook_token, created_by)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7, $8)
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
}
