use std::sync::LazyLock;

use anyhow::Context;
use oauth2_reqwest::ReqwestClient;
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

pub static OAUTH2_REQWEST: LazyLock<ReqwestClient> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client should build")
        .into()
});

pub static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client should build")
});
