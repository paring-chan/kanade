use poem_openapi::OpenApi;

#[OpenApi(prefix_path = "/agent/steps", tag = "ApiTags::Agent")]
pub struct AgentJobStepsApi;

#[OpenApi(prefix_path = "/agent/steps", tag = "ApiTags::Agent")]
impl AgentJobStepsApi {
}
