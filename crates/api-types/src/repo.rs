use chrono::{DateTime, Utc};
use garde::Validate;
use poem_openapi::{payload::Json, ApiResponse, Object};
use uuid::Uuid;

use crate::{ErrorResponse, TeamResponse};

#[derive(Debug, Object, Validate)]
#[oai(rename_all = "camelCase")]
pub struct RepoCreateRequest {
    #[garde(skip)]
    pub team_id: Uuid,

    #[garde(length(min = 2, max = 20))]
    pub name: String,
    #[garde(pattern("^[a-zA-Z0-9-_]{3,20}$"), length(min = 3, max = 20))]
    pub slug: String,

    #[garde(skip)]
    pub forge_repo_id: String,
    #[garde(skip)]
    pub forge_id: Uuid,
}

#[derive(Debug, ApiResponse)]
pub enum RepoCreateEndpointResponse {
    #[oai(status = 200)]
    Ok(Json<RepoCreateResponse>),
    #[oai(status = 400)]
    ValidationFailed(Json<serde_json::Value>),
    #[oai(status = 401)]
    Conflict(Json<ErrorResponse>),
}

#[derive(Debug, Object)]
#[oai(rename_all = "camelCase")]
pub struct RepoCreateResponse {
    pub id: Uuid,
    pub repo_slug: String,
    pub team_slug: String,
}

#[derive(Debug, ApiResponse)]
pub enum GetRepoResponse {
    #[oai(status = 200)]
    Ok(Json<RepoResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

#[derive(Debug, Object)]
pub struct RepoResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub team: TeamResponse,
}
