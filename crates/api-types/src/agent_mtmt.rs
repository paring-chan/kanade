use poem_openapi::{payload::Json, ApiResponse, Object};
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
