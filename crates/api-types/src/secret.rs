use chrono::{DateTime, Utc};
use garde::Validate;
use poem_openapi::{payload::Json, ApiResponse, Object};
use uuid::Uuid;

use crate::{ApiEventType, ErrorResponse};

#[derive(Object)]
pub struct SecretInfo {
    pub id: Uuid,
    pub key: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Object, Validate)]
pub struct SecretCreateRequest {
    #[garde(length(min = 1, max = 36))]
    pub key: String,
    #[garde(length(min = 6, max = 300))]
    pub value: String,
    #[garde(length(min = 1, max = 6))]
    pub scopes: Vec<ApiEventType>,
}

#[derive(ApiResponse)]
pub enum SecretCreateEndpointResponse {
    #[oai(status = 200)]
    Ok,
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
    #[oai(status = 400)]
    ValidationFailed(Json<serde_json::Value>),
    #[oai(status = 409)]
    Conflict(Json<ErrorResponse>),
}
