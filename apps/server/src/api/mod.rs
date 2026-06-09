use poem_openapi::{OpenApi, Tags};

use crate::api::{forge::ForgeApi, user::UserApi};

mod agent;
mod forge;
mod security;
mod user;

#[derive(Tags)]
enum ApiTags {
    /// Agent 전용 API
    Agent,
    /// 유저 정보 API
    User,
    /// 포지 API
    Forge,
}

pub fn api() -> impl OpenApi {
    (UserApi, ForgeApi, agent::api())
}
