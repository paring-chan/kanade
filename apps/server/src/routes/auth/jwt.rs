use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginClaims {
    pub forge_id: String,
    pub aud: String,
    pub exp: u64,
    pub iss: String,
}
