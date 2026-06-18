use chrono::{DateTime, Utc};
use poem_openapi::{payload::Json, ApiResponse, Enum, Object};
use uuid::Uuid;

use crate::ErrorResponse;

#[derive(Debug, Object)]
#[oai(rename_all = "camelCase")]
pub struct AgentCreateRequest {
    pub name: String,
    pub team_id: Uuid,
}

#[derive(Debug, ApiResponse)]
pub enum AgentCreateEndpointResponse {
    #[oai(status = 200)]
    Ok(Json<AgentCreateResponse>),
    #[oai(status = 400)]
    ValidationFailed(Json<serde_json::Value>),
    #[oai(status = 401)]
    Conflict(Json<ErrorResponse>),
}

#[derive(Debug, Object)]
pub struct AgentCreateResponse {
    pub id: Uuid,
    pub name: String,

    pub token: String,
}

#[derive(Debug, Enum)]
#[oai(rename_all = "camelCase")]
pub enum AgentStatusResponse {
    Idle,
    Busy,
    Offline,
}

#[derive(Debug, Object)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub status: AgentStatusResponse,
    pub is_global: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
}

#[derive(Debug, ApiResponse)]
pub enum DeleteAgentEndpointResponse {
    #[oai(status = 203)]
    Ok(Json<DeleteAgentResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

#[derive(Debug, Object)]
pub struct DeleteAgentResponse {
    pub message: String,
}
