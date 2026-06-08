use poem_openapi::OpenApi;

use crate::api::agent::jobs::AgentJobsApi;

mod jobs;
mod types;

pub fn api() -> impl OpenApi {
    AgentJobsApi
}
