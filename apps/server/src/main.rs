use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use poem::{Route, Server, handler, listener::TcpListener};
use serde::Deserialize;

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

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    4000
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();

    let figment = Figment::new()
        .merge(Toml::file(&args.config))
        .merge(Env::prefixed("KANADE_SERVER_").split("__"));

    let config: AppConfig = figment.extract().expect("Failed to extract config");

    debug!("config: {config:?}");

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr);
    let app = Route::new().at("/", index);

    Server::new(listener)
        .name("kanade-server")
        .run(app)
        .await
        .expect("Failed to start server");
}
