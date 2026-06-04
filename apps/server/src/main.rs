mod config;

use anyhow::Context as _;
use clap::Parser;
use config::AppConfig;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use poem::{Route, Server, handler, listener::TcpListener};
use secrecy::ExposeSecret;
use sqlx::{Database, PgPool};

#[macro_use]
extern crate tracing;

#[handler]
fn index() -> &'static str {
    "hello world!"
}

#[derive(Parser, Debug)]
#[command(name = "kanade-server", version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        global = true,
        env = "KANADE_SERVER_CONFIG",
        default_value = "kanade-server.toml"
    )]
    pub config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();

    let figment = Figment::new()
        .merge(Toml::file(&args.config))
        .merge(Env::prefixed("KANADE_SERVER_").split("__"));

    let config: AppConfig = figment.extract().context("Failed to extract config")?;

    debug!("config: {config:?}");

    let db = PgPool::connect(config.db.url.expose_secret())
        .await
        .context("failed to connect to db")?;

    let migrator = sqlx::migrate!("./migrations");

    migrator.run(&db).await.context("failed to run migration")?;

    info!("connected to db");

    let listener = TcpListener::bind(&config.server.bind);
    let app = Route::new().at("/", index);

    Server::new(listener)
        .name("kanade-server")
        .run(app)
        .await
        .context("failed to start server")?;

    Ok(())
}
