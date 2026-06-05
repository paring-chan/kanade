use chrono::{DateTime, Utc};
use sqlx::{
    prelude::{FromRow, Type},
    types::Json,
};
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

#[derive(Debug, Type)]
#[sqlx(type_name = "event_type", rename_all = "snake_case")]
pub enum EventType {
    Push,
    Tag,
    Release,
    PullRequest,
    Cron,
    Manual,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "pipeline_status", rename_all = "snake_case")]
pub enum PipelineStatus {
    Evaluating,
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Waiting,
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
}
