use std::sync::Arc;

use clap::Parser;
use secrecy::{ExposeSecret, SecretString};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    data::{
        forges::{ForgeConfig, ForgejoForgeConfig},
        rows::ForgeRow,
    },
    util::open_db,
};

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    Add {
        #[command(subcommand)]
        subcommand: AddSubcommand,

        #[arg(long)]
        name: String,
    },
    List(List),
    Delete(Delete),
    Configure(Configure),
}

#[derive(clap::Subcommand, Debug)]
pub enum AddSubcommand {
    Forgejo(Forgejo),
}

#[derive(Parser, Debug)]
pub struct Forgejo {
    #[arg(long)]
    pub url: String,

    #[arg(long)]
    pub client_id: String,
    #[arg(long)]
    pub client_secret: SecretString,
}

#[derive(Parser, Debug)]
pub struct List {}

#[derive(Parser, Debug)]
pub struct Delete {}

#[derive(Parser, Debug)]
pub struct Configure {}

pub async fn run(config: Arc<AppConfig>, subcommand: Subcommand) -> anyhow::Result<()> {
    match subcommand {
        Subcommand::Add { subcommand, name } => add(config, name, subcommand).await,
        Subcommand::List(_) => list(config).await,
        Subcommand::Delete(_) => todo!(),
        Subcommand::Configure(_) => todo!(),
    }
}

async fn list(config: Arc<AppConfig>) -> anyhow::Result<()> {
    let db = open_db(&config).await?;

    let result = sqlx::query_as::<_, ForgeRow>(r#"SELECT * FROM forge ORDER BY created_at"#)
        .fetch_all(&db)
        .await?;

    for item in result {
        println!(
            "---\nid: {}\nname: {}\ncreated at: {}\nupdated at: {}\nconfig:\n{}",
            item.id,
            item.name,
            item.created_at,
            item.updated_at,
            serde_json::to_string_pretty(&item.config.0)?
        );
    }

    Ok(())
}

async fn add(
    config: Arc<AppConfig>,
    name: String,
    subcommand: AddSubcommand,
) -> anyhow::Result<()> {
    let forge_config = match subcommand {
        AddSubcommand::Forgejo(forgejo) => ForgeConfig::Forgejo(ForgejoForgeConfig {
            url: forgejo.url,
            client_id: forgejo.client_id,
            client_secret: forgejo.client_secret.expose_secret().to_string(),
        }),
    };

    let db = open_db(&config).await?;

    let id = Uuid::new_v4();

    sqlx::query(r#"INSERT INTO "forge" (id, name, config) VALUES ($1, $2, $3)"#)
        .bind(id)
        .bind(name)
        .bind(Json(forge_config))
        .execute(&db)
        .await?;

    info!("added");

    Ok(())
}
