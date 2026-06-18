use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub api_uri: String,
    pub token: SecretString,
}
