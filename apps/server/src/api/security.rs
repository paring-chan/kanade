use std::sync::{Arc, LazyLock};

use jsonwebtoken::Validation;
use poem::Request;
use poem_openapi::{SecurityScheme, auth::ApiKey};
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
