use std::sync::Arc;

use anyhow::Context;
use poem::{Route, Server, listener::TcpListener};

use crate::config::AppConfig;

pub async fn run(config: Arc<AppConfig>) -> anyhow::Result<()> {
    info!("connected to db");

    let listener = TcpListener::bind(&config.server.bind);
    let app = Route::new().at("/", crate::index);

    Server::new(listener)
        .name("kanade-server")
        .run(app)
        .await
        .context("failed to start server")?;

    Ok(())
}
