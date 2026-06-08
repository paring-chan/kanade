use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub api_uri: String,
}
