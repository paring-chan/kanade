use std::sync::Arc;

use anyhow::Context;

use crate::{config::AppConfig, util::open_db};

pub async fn run(config: Arc<AppConfig>) -> anyhow::Result<()> {
    let db = open_db(&config).await?;

    let migrator = sqlx::migrate!("./migrations");

    migrator.run(&db).await.context("failed to run migration")?;

    Ok(())
}
