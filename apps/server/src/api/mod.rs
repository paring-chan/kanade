use poem_openapi::{OpenApi, Tags};

mod agent;

#[derive(Tags)]
enum ApiTags {
    /// Agent 전용 API
    Agent,
}

pub fn api() -> impl OpenApi {
    agent::api()
}
