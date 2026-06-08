mod agent;
mod config;

use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser;
use config::AgentConfig;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

#[macro_use]
extern crate tracing;

#[derive(Parser, Debug)]
#[command(name = "kanade-agent", version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        global = true,
        env = "KANADE_AGENT_CONFIG",
        default_value = "kanade-agent.toml"
    )]
    pub config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let args = Args::parse();

    let figment = Figment::new()
        .merge(Toml::file(&args.config))
        .merge(Env::prefixed("KANADE_AGENT__").split("__"));

    let config: Arc<AgentConfig> = Arc::new(figment.extract().context("Failed to extract config")?);

    debug!("config: {config:?}");

    let agent = Arc::new(agent::KanadeAgent::new(config));
    agent.run().await;

    Ok(())
}
