use poem_openapi::OpenApi;

use crate::api::agent::jobs::AgentJobsApi;
use crate::api::agent::steps::AgentJobStepsApi;

pub mod jobs;
pub mod steps;

pub fn api() -> impl OpenApi {
    (AgentJobsApi, AgentJobStepsApi)
}
