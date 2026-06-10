use chrono::{DateTime, Utc};
use garde::Validate;
use poem_openapi::{payload::Json, ApiResponse, Object};
use uuid::Uuid;

use crate::ErrorResponse;

#[derive(Debug, Object, Validate)]
pub struct TeamCreateRequest {
    #[garde(length(min = 2, max = 20))]
    pub name: String,
    #[garde(pattern("^[a-zA-Z0-9-_]{3,20}$"), length(min = 3, max = 20))]
    pub slug: String,
}

#[derive(Debug, ApiResponse)]
pub enum TeamCreateEndpointResponse {
    #[oai(status = 200)]
    Ok(Json<TeamResponse>),
    #[oai(status = 400)]
    ValidationFailed(Json<serde_json::Value>),
    #[oai(status = 401)]
    Conflict(Json<ErrorResponse>),
}

#[derive(Debug, Object)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
