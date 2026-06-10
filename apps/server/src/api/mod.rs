use poem_openapi::{OpenApi, Tags};

use crate::api::{forge::ForgeApi, team::TeamApi, user::UserApi};

mod agent;
mod forge;
mod security;
mod team;
mod user;

#[derive(Tags)]
enum ApiTags {
    /// Agent 전용 API
    Agent,
    /// 유저 정보 API
    User,
    /// 포지 API
    Forge,
    /// 팀 API
    Team,
}

pub fn api() -> impl OpenApi {
    (UserApi, TeamApi, ForgeApi, agent::api())
}
