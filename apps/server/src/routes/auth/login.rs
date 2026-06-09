use std::{ops::Add, sync::Arc};

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header};
use oauth2::CsrfToken;
use poem::{
    IntoResponse, Response, handler,
    web::{Data, Path, Redirect},
};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{
    config::{AppConfig, JwtSigningKey},
    data::forges::ForgeConfig,
    routes::auth::jwt::LoginClaims,
    util::{AUD_LOGIN, JWT_ISS},
};

#[handler]
pub async fn login(
    forge_id: Path<Uuid>,
    Data(db): Data<&PgPool>,
    signing_key: Data<&Arc<JwtSigningKey>>,
    config: Data<&Arc<AppConfig>>,
) -> crate::Result<Response> {
    #[derive(FromRow)]
    struct ForgeRow {
        pub config: sqlx::types::Json<ForgeConfig>,
    }

    let forge = sqlx::query_as::<_, ForgeRow>(r#"SELECT config FROM forge WHERE id = $1"#)
        .bind(*forge_id)
        .fetch_optional(db)
        .await?
        .ok_or(crate::AppError::UnknownForge(*forge_id))?;

    match forge.config.0 {
        ForgeConfig::Forgejo(forgejo) => {
            let client = forgejo.oauth2_client(&config)?;

            let header = Header {
                alg: Algorithm::HS512,
                ..Default::default()
            };
            let claims = LoginClaims {
                forge_id: forge_id.to_string(),
                exp: Utc::now().add(Duration::minutes(5)).timestamp() as _,
                aud: AUD_LOGIN.to_string(),
                iss: JWT_ISS.to_string(),
            };

            let token = jsonwebtoken::encode(&header, &claims, signing_key.encoding())?;

            let (url, _) = client.authorize_url(|| CsrfToken::new(token)).url();

            Ok(Redirect::see_other(url).into_response())
        }
    }
}
