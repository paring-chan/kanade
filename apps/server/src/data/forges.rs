use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ForgeConfig {
    Forgejo(ForgejoForgeConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgejoForgeConfig {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
}
