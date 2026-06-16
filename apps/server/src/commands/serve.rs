use std::sync::Arc;

use anyhow::Context;
use hex::FromHex;
use poem::{EndpointExt, Server, listener::TcpListener};
use secrecy::ExposeSecret;

use crate::{
    config::{AppConfig, JwtSigningKey},
    crypto::CryptoEngine,
    forges::AllForges,
    realtime::Realtime,
    routes::routes,
    util::open_db,
};

pub async fn run(config: Arc<AppConfig>) -> anyhow::Result<()> {
    let db = open_db(&config).await?;
    info!("connected to db");

    let listener = TcpListener::bind(&config.server.bind);

    let crypto = Arc::new(CryptoEngine::new(
        Vec::<u8>::from_hex(config.encryption_key.expose_secret())?.into(),
    )?);

    let _realtime = Arc::new(Realtime::new(config.clone()).await?);

    let app = routes()
        .data(db.clone())
        .data(Arc::new(JwtSigningKey::new(
            config.jwt_secret.expose_secret().as_bytes(),
        )))
        .data(config.clone())
        .data(crypto.clone())
        .data(Arc::new(AllForges::new(config.clone(), crypto, db)?));

    Server::new(listener)
        .name("kanade-server")
        .run(app)
        .await
        .context("failed to start server")?;

    Ok(())
}
