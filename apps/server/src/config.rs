use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    pub db: DbConfig,
    pub encryption_key: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub url: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
        }
    }
}

fn default_bind() -> String {
    "0.0.0.0:4000".to_string()
}
