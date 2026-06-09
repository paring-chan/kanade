use chrono::{DateTime, Utc};
use poem_openapi::{payload::Json, ApiResponse, Object};
use uuid::Uuid;

#[derive(ApiResponse)]
pub enum UserEndpointResponse {
    #[oai(status = 200)]
    Ok(Json<UserResponse>),
}

#[derive(Object)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub nick: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
