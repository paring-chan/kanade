use jsonwebtoken::{DecodingKey, EncodingKey};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DbConfig,
    pub encryption_key: SecretString,
    pub jwt_secret: SecretString,
    pub valkey: ValkeyConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowConfig {
    pub default_image: String,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            default_image: "oci.pari.ng/kanade/build-env:0.1".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ValkeyConfig {
    pub url: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub url: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,

    pub public_url: String,
}

fn default_bind() -> String {
    "0.0.0.0:4000".to_string()
}

pub struct JwtSigningKey {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtSigningKey {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    pub fn encoding(&self) -> &EncodingKey {
        &self.encoding
    }

    pub fn decoding(&self) -> &DecodingKey {
        &self.decoding
    }
}
