use std::sync::Arc;

use anyhow::Context;
use poem::{EndpointExt, Server, listener::TcpListener};
use secrecy::ExposeSecret;

use crate::{
    config::{AppConfig, JwtSigningKey},
    routes::routes,
    util::open_db,
};

pub async fn run(config: Arc<AppConfig>) -> anyhow::Result<()> {
    let db = open_db(&config).await?;
    info!("connected to db");

    let listener = TcpListener::bind(&config.server.bind);
    let app = routes()
        .data(db)
        .data(Arc::new(JwtSigningKey::new(
            config.jwt_secret.expose_secret().as_bytes(),
        )))
        .data(config.clone());

    Server::new(listener)
        .name("kanade-server")
        .run(app)
        .await
        .context("failed to start server")?;

    Ok(())
}
