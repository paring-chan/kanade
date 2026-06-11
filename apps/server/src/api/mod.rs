use poem_openapi::{OpenApi, Tags};

use crate::api::{forge::ForgeApi, repo::RepoApi, team::TeamApi, user::UserApi};

mod agent;
mod forge;
mod repo;
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
    /// 저장소 API
    Repo,
}

pub fn api() -> impl OpenApi {
    (UserApi, TeamApi, ForgeApi, RepoApi, agent::api())
}
