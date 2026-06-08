use poem_openapi::OpenApi;

use crate::api::agent::jobs::AgentJobsApi;

pub mod jobs;

pub fn api() -> impl OpenApi {
    AgentJobsApi
}
