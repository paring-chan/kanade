use chrono::{DateTime, Utc};
use poem_openapi::{payload::Json, ApiResponse, Object};
use uuid::Uuid;

use crate::ForgeInfoResponse;

#[derive(ApiResponse)]
pub enum UserEndpointResponse {
    #[oai(status = 200)]
    Ok(Json<UserResponse>),
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub nick: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
pub struct UserForgeResponse {
    pub id: Uuid,
    pub forge: ForgeInfoResponse,
    pub forge_user_id: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
