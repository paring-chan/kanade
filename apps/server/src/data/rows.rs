use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, types::Json};
use uuid::Uuid;

use crate::data::forges::ForgeConfig;

#[derive(Debug, FromRow)]
pub struct ForgeRow {
    pub id: Uuid,
    pub name: String,
    pub config: Json<ForgeConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
