use poem_openapi::{OpenApi, Tags};

mod agent;
mod forge;
mod jobs;
mod pipeline;
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
    /// 파이프라인 API
    Pipeline,
}

pub fn api() -> impl OpenApi {
    (
        pipeline::PipelineApi,
        jobs::JobsApi,
        user::UserApi,
        team::TeamApi,
        forge::ForgeApi,
        repo::RepoApi,
        agent::api(),
    )
}
