use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTokenClaims {
    pub sub: Uuid,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
}
