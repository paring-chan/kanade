use poem_openapi::{OpenApi, Tags};

mod agent;
mod agent_mgmt;
mod forge;
mod jobs;
mod pipeline;
mod repo;
mod security;
mod team;
mod user;

#[derive(Tags)]
enum ApiTags {
    /// 에이전트 전용 API
    Agent,
    /// 에이전트 관리 API
    AgentManagement,
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
        agent_mgmt::AgentManagementApi,
        user::UserApi,
        team::TeamApi,
        forge::ForgeApi,
        repo::RepoApi,
        agent::api(),
    )
}
