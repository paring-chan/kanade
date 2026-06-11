use std::sync::Arc;

use api_types::{
    ErrorResponse, ProjectCreateEndpointResponse, ProjectCreateRequest, ProjectCreateResponse,
};
use garde::Validate;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
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
        Json(payload): Json<ProjectCreateRequest>,
        Data(forges): Data<&Arc<AllForges>>,
        Data(crypto): Data<&Arc<CryptoEngine>>,
    ) -> poem::Result<ProjectCreateEndpointResponse> {
        self._create(db, user_id, payload, forges, crypto)
            .await
            .map_err(Into::into)
    }

    async fn _create(
        &self,
        db: &PgPool,
        user_id: Uuid,
        payload: ProjectCreateRequest,
        forges: &AllForges,
        crypto: &Arc<CryptoEngine>,
    ) -> crate::Result<ProjectCreateEndpointResponse> {
        match payload.validate() {
            Ok(_) => {}
            Err(report) => {
                return Ok(ProjectCreateEndpointResponse::ValidationFailed(Json(
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
            return Ok(ProjectCreateEndpointResponse::Conflict(Json(
                ErrorResponse {
                    message: "해당 팀 내에 이미 동일한 슬러그를 가진 프로젝트가 존재합니다.".into(),
                },
            )));
        }
        if exists.1 {
            return Ok(ProjectCreateEndpointResponse::Conflict(Json(
                ErrorResponse {
                    message: "이 저장소는 이미 다른 프로젝트에 연동되어 있습니다.".into(),
                },
            )));
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

        Ok(ProjectCreateEndpointResponse::Ok(Json(
            ProjectCreateResponse {
                id: res.id,
                repo_slug: res.repo_slug,
                team_slug: res.team_slug,
            },
        )))
    }
}
