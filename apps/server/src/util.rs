use anyhow::Context;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::config::AppConfig;

pub async fn open_db(config: &AppConfig) -> anyhow::Result<PgPool> {
    let db = PgPool::connect(config.db.url.expose_secret())
        .await
        .context("failed to connect to db")?;

    Ok(db)
}

pub const JWT_ISS: &str = "kanade";
pub const AUD_LOGIN: &str = "login";
pub const AUD_USER: &str = "user";
