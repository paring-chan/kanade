use std::sync::{Arc, LazyLock};

use jsonwebtoken::Validation;
use poem::Request;
use poem_openapi::{SecurityScheme, auth::ApiKey};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::UserTokenClaims,
    config::JwtSigningKey,
    util::{AUD_USER, JWT_ISS},
};

static USER_JWT_VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut v = Validation::new(jsonwebtoken::Algorithm::HS512);
    v.set_issuer(&[JWT_ISS]);
    v.set_audience(&[AUD_USER]);
    v
});

#[derive(SecurityScheme)]
#[oai(
    ty = "api_key",
    key_in = "header",
    key_name = "Authorization",
    checker = "Self::check"
)]
pub struct ApiKeyAuth(pub Uuid);

impl ApiKeyAuth {
    async fn check(req: &Request, api_key: ApiKey) -> Option<Uuid> {
        let key = req.data::<Arc<JwtSigningKey>>().unwrap();
        let token = jsonwebtoken::decode::<UserTokenClaims>(
            api_key.key.strip_prefix("Bearer ")?,
            key.decoding(),
            &USER_JWT_VALIDATION,
        )
        .ok()?;

        Some(token.claims.sub)
    }
}

#[derive(SecurityScheme)]
#[oai(
    ty = "api_key",
    key_in = "header",
    key_name = "X-Agent-Token",
    checker = "Self::check"
)]
pub struct AgentTokenAuth(pub Uuid);

impl AgentTokenAuth {
    async fn check(req: &Request, api_key: ApiKey) -> Option<Uuid> {
        let db = req.data::<PgPool>().unwrap();

        let mut hasher = Sha256::new();
        hasher.update(api_key.key.as_bytes());
        let hash_bytes = hasher.finalize().to_vec();

        let agent = sqlx::query!("SELECT id FROM agent WHERE token_sha256 = $1", &hash_bytes)
            .fetch_optional(db)
            .await
            .ok()??;

        Some(agent.id)
    }
}
