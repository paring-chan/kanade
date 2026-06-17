use api_types::{
    AgentCreateEndpointResponse, AgentCreateRequest, AgentCreateResponse, AgentResponse,
};
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
    #[oai(path = "/", method = "get")]
    async fn list_agents(
        &self,
        Data(db): Data<&PgPool>,
        ApiKeyAuth(user_id): ApiKeyAuth,
    ) -> poem::Result<Json<Vec<AgentResponse>>> {
        self._list_agents(db, user_id)
            .await
            .map(Json)
            .map_err(Into::into)
    }

    async fn _list_agents(&self, db: &PgPool, user_id: Uuid) -> crate::Result<Vec<AgentResponse>> {
        let mut tx = db.begin_as(user_id).await?;

        // TODO: filter owned
        let results = sqlx::query!(
            r#"
            SELECT
                id,
                name,
                status as "status: AgentStatus",
                is_global,
                created_at,
                updated_at,
                last_heartbeat_at
            FROM agent
            "#
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(results
            .into_iter()
            .map(|x| AgentResponse {
                id: x.id,
                name: x.name,
                status: x.status.into(),
                is_global: x.is_global,
                created_at: x.created_at,
                updated_at: x.updated_at,
                last_heartbeat_at: x.last_heartbeat_at,
            })
            .collect())
    }

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
