use api_types::{AgentCreateEndpointResponse, AgentCreateRequest, AgentCreateResponse};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::security::ApiKeyAuth, data::db::AgentStatus, error::AppError,
    security::DatabaseSecurityExt,
};

pub struct AgentManagementApi;

#[OpenApi(prefix_path = "/agents", tag = "super::ApiTags::AgentManagement")]
impl AgentManagementApi {
    #[oai(path = "/", method = "post")]
    async fn create_agent(
        &self,
        Data(db): Data<&PgPool>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Json(payload): Json<AgentCreateRequest>,
    ) -> poem::Result<AgentCreateEndpointResponse> {
        self._create_agent(db, user_id, payload)
            .await
            .map_err(Into::into)
    }

    async fn _create_agent(
        &self,
        db: &PgPool,
        user_id: Uuid,
        payload: AgentCreateRequest,
    ) -> crate::Result<AgentCreateEndpointResponse> {
        let mut raw_token = [0u8; 64];
        SysRng
            .try_fill_bytes(&mut raw_token)
            .map_err(|e| AppError::InternalError(e.into()))?;
        let token = hex::encode(&raw_token);
        let mut hasher = Sha256::new();
        hasher.update(&token.as_bytes());
        let token_sha256 = hasher.finalize().to_vec();

        let mut tx = db.begin_as(user_id).await?;

        let is_global = payload.team_id.is_nil();

        let result = sqlx::query!(
            r#"
            INSERT INTO agent
                (id, name, status, token_sha256, is_global)
            VALUES
                ($1, $2, $3, $4, $5)
            RETURNING id, name
            "#,
            Uuid::new_v4(),
            payload.name,
            AgentStatus::Offline as AgentStatus,
            token_sha256,
            is_global,
        )
        .fetch_one(&mut *tx)
        .await?;

        // TODO: add to team(ddl not added)

        tx.commit().await?;

        Ok(AgentCreateEndpointResponse::Ok(Json(AgentCreateResponse {
            id: result.id,
            name: result.name,
            token,
        })))
    }
}
