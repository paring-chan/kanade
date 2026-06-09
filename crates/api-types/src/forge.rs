use poem_openapi::{payload::Json, ApiResponse, Object};
use uuid::Uuid;

#[derive(Debug, ApiResponse)]
pub enum ForgeInfoEndpointResponse {
    /// 정상 응답
    #[oai(status = 200)]
    Ok(Json<Vec<ForgeInfoResponse>>),
}

#[derive(Debug, Object)]
pub struct ForgeInfoResponse {
    /// 포지 ID
    pub id: Uuid,
    pub name: String,
}
