mod config;

use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser;
use config::AppConfig;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use poem::handler;

#[macro_use]
extern crate tracing;

pub use api::api;
pub use realtime::types::{EventMessage, LogEntry};

mod api;
mod auth;
mod commands;
mod crypto;
mod data;
mod error;
mod forges;
mod realtime;
mod routes;
mod security;
mod util;

use error::*;

#[derive(Parser, Debug)]
pub enum Command {
    Serve,
    Migrate,
    Forge {
        #[command(subcommand)]
        subcommand: crate::commands::ForgesSubcommand,
    },
}

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
    #[command(subcommand)]
    pub command: Command,
}

pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();

    let figment = Figment::new()
        .merge(Toml::file(&args.config))
        .merge(Env::prefixed("KANADE_SERVER__").split("__"));

    let config: Arc<AppConfig> = Arc::new(figment.extract().context("Failed to extract config")?);

    debug!("config: {config:?}");

    match args.command {
        Command::Serve => commands::serve::run(config).await?,
        Command::Migrate => commands::migrate::run(config).await?,
        Command::Forge { subcommand } => commands::forges::run(config, subcommand).await?,
    };

    Ok(())
}
