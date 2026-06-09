use std::{
    ops::Add,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, Header, TokenData, Validation};
use oauth2::{AuthorizationCode, CsrfToken, TokenResponse};
use oauth2_reqwest::ReqwestClient;
use poem::{
    IntoResponse, Response, handler,
    web::{Data, Query, Redirect},
};
use reqwest::header::AUTHORIZATION;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{
    auth::UserTokenClaims,
    config::{AppConfig, JwtSigningKey},
    crypto::CryptoEngine,
    data::forges::ForgeConfig,
    error::AppError,
    routes::auth::jwt::LoginClaims,
    util::{AUD_LOGIN, AUD_USER, JWT_ISS},
};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum CallbackQuery {
    Successful {
        code: AuthorizationCode,
        state: CsrfToken,
    },
    Failed {
        error: String,
        error_description: Option<String>,
    },
}

#[handler]
pub async fn callback(
    Query(query): Query<CallbackQuery>,
    Data(db): Data<&PgPool>,
    Data(signing_key): Data<&Arc<JwtSigningKey>>,
    Data(config): Data<&Arc<AppConfig>>,
    Data(crypto_engine): Data<&Arc<CryptoEngine>>,
) -> crate::Result<Response> {
    match query {
        CallbackQuery::Successful { code, state } => {
            let token = match jsonwebtoken::decode::<LoginClaims>(
                state.into_secret(),
                signing_key.decoding(),
                &{
                    let mut val = Validation::new(jsonwebtoken::Algorithm::HS512);
                    val.set_issuer(&[JWT_ISS]);
                    val.set_audience(&[AUD_LOGIN]);
                    val
                },
            ) {
                Ok(claims) => claims,
                Err(_) => {
                    return Ok(Redirect::see_other(format!(
                        "/login?error={}",
                        urlencoding::encode("잘못된 인증 토큰입니다. 다시  시도해주세요.")
                    ))
                    .into_response());
                }
            };

            match process_callback(code, token, db, config, &signing_key, &crypto_engine).await {
                Err(err) => {
                    return Ok(Redirect::see_other(format!(
                        "/login?error={}",
                        urlencoding::encode(&format!("인증에 실패했습니다: {err}"))
                    ))
                    .into_response());
                }
                res => res,
            }
        }
        CallbackQuery::Failed {
            error,
            error_description,
        } => Ok(Redirect::see_other(format!(
            "/login?error={}",
            urlencoding::encode(&error_description.unwrap_or(error))
        ))
        .into_response()),
    }
}

static OAUTH2_REQWEST: LazyLock<ReqwestClient> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client should build")
        .into()
});

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client should build")
});

#[instrument(skip(code, token, db, config, signing_key, crypto_engine), err(Debug))]
async fn process_callback(
    code: AuthorizationCode,
    token: TokenData<LoginClaims>,
    db: &PgPool,
    config: &AppConfig,
    signing_key: &JwtSigningKey,
    crypto_engine: &CryptoEngine,
) -> crate::Result<Response> {
    let forge_id =
        Uuid::from_str(&token.claims.forge_id).map_err(|e| AppError::InternalError(e.into()))?;

    #[derive(FromRow)]
    struct ForgeRow {
        pub config: sqlx::types::Json<ForgeConfig>,
    }

    let forge = sqlx::query_as::<_, ForgeRow>(r#"SELECT config FROM forge WHERE id = $1"#)
        .bind(forge_id)
        .fetch_optional(db)
        .await?
        .ok_or(crate::AppError::UnknownForge(forge_id))?;

    let result = match forge.config.0 {
        ForgeConfig::Forgejo(forgejo) => {
            let client = forgejo.oauth2_client(config)?;
            let token_result = client
                .exchange_code(code)
                .request_async(&*OAUTH2_REQWEST)
                .await
                .map_err(|e| AppError::InternalError(e.into()))?;

            let access_token = token_result.access_token().clone().into_secret();
            let refresh_token = token_result
                .refresh_token()
                .ok_or(AppError::InvalidTokenResponse)?
                .clone()
                .into_secret();
            let access_token_expires_at = Utc::now().add(
                token_result
                    .expires_in()
                    .ok_or(AppError::InvalidTokenResponse)?,
            );

            #[derive(Debug, Deserialize)]
            struct ForgejoProfile {
                pub id: u32,
                pub login: String,
                /// empty if none
                pub full_name: String,
                /// empty if none
                pub avatar_url: String,
                pub email: String,
            }

            let ForgejoProfile {
                id,
                login,
                full_name,
                avatar_url,
                email,
            } = HTTP
                .get(format!("{}/api/v1/user", &forgejo.url))
                .header(AUTHORIZATION, format!("token {access_token}"))
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map_err(|e| AppError::InternalError(e.into()))?
                .json::<ForgejoProfile>()
                .await
                .map_err(|e| AppError::InternalError(e.into()))?;

            let full_name = Some(full_name).filter(|x| !x.is_empty());
            let avatar_url = Some(avatar_url).filter(|x| !x.is_empty());

            ForgeAuthResult {
                access_token: SecretString::from(access_token),
                refresh_token: SecretString::from(refresh_token),
                access_token_expires_at,
                avatar_url,
                sub: id.to_string(),
                nick: full_name.unwrap_or_else(|| login.clone()),
                email: Some(email),
                login,
            }
        }
    };

    let access_token = crypto_engine.encrypt(result.access_token.expose_secret())?;
    let refresh_token = crypto_engine.encrypt(result.refresh_token.expose_secret())?;

    let mut tx = db.begin().await?;

    #[derive(FromRow)]
    struct UserIdRow {
        user_id: Uuid,
    }

    let existing_user = sqlx::query_as::<_, UserIdRow>(
        r#"
        SELECT u.id as user_id
        FROM "user" u
        LEFT JOIN user_forge uf ON u.id = uf.user_id
        WHERE
            uf.forge_id = $1 AND
            uf.forge_user_id = $2
        FOR UPDATE OF u
    "#,
    )
    .bind(forge_id)
    .bind(&result.sub)
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = match existing_user {
        Some(row) => {
            sqlx::query(
                r#"
                UPDATE user_forge
                SET
                    access_token = $1,
                    refresh_token = $2,
                    access_token_expires_at = $3
                WHERE forge_id = $4 AND user_id = $5
                "#,
            )
            .bind(&access_token)
            .bind(&refresh_token)
            .bind(result.access_token_expires_at)
            .bind(forge_id)
            .bind(row.user_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                UPDATE "user"
                SET nick = $1, avatar_url = $2
                WHERE id = $3
                "#,
            )
            .bind(result.nick)
            .bind(result.avatar_url)
            .bind(row.user_id)
            .execute(&mut *tx)
            .await?;

            row.user_id
        }
        None => {
            let user_id = Uuid::new_v4();

            #[derive(FromRow)]
            struct UserInsertResult {
                id: Uuid,
            }

            let user_insert_query = r#"
                INSERT INTO "user"
                    (id, username, nick, email, avatar_url)
                VALUES
                    ($1, $2, $3, $4, $5)
                ON CONFLICT (email) DO UPDATE SET
                    email = EXCLUDED.email,
                    nick = EXCLUDED.nick,
                    avatar_url = EXCLUDED.avatar_url
                RETURNING id
            "#;

            let user_id = match sqlx::query_as::<_, UserInsertResult>(user_insert_query)
                .bind(user_id)
                .bind(&result.login)
                .bind(&result.nick)
                .bind(&result.email)
                .bind(&result.avatar_url)
                .fetch_one(&mut *tx)
                .await
            {
                Ok(r) => r.id,
                // 유저네임 겹침
                Err(e)
                    if e.as_database_error()
                        .map(|x| {
                            x.is_unique_violation() && x.constraint() == Some("user_username_key")
                        })
                        .unwrap_or_default() =>
                {
                    sqlx::query_as::<_, UserInsertResult>(user_insert_query)
                        .bind(user_id)
                        .bind(format!("{}_{}", result.login, nanoid::nanoid!(6)))
                        .bind(&result.nick)
                        .bind(&result.email)
                        .bind(&result.avatar_url)
                        .fetch_one(&mut *tx)
                        .await?
                        .id
                }
                Err(e) => return Err(e.into()),
            };

            sqlx::query(
                r#"
                INSERT INTO user_forge
                    (id, user_id, forge_id, forge_user_id, access_token, refresh_token, access_token_expires_at)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(forge_id)
            .bind(&result.sub)
            .bind(&access_token)
            .bind(&refresh_token)
            .bind(result.access_token_expires_at)
            .execute(&mut *tx).await?;

            user_id
        }
    };

    let header = Header {
        alg: Algorithm::HS512,
        ..Default::default()
    };
    let claims = UserTokenClaims {
        sub: user_id,
        exp: Utc::now().add(Duration::hours(8)).timestamp() as _,
        aud: AUD_USER.to_string(),
        iss: JWT_ISS.to_string(),
    };

    let token = jsonwebtoken::encode(&header, &claims, signing_key.encoding())?;

    tx.commit().await?;

    Ok(Redirect::see_other(format!(
        "/login/success?token={}",
        urlencoding::encode(&token)
    ))
    .into_response())
}

#[derive(Debug)]
struct ForgeAuthResult {
    pub access_token: SecretString,
    pub refresh_token: SecretString,
    pub access_token_expires_at: DateTime<Utc>,

    pub sub: String,
    pub login: String,
    pub nick: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}
