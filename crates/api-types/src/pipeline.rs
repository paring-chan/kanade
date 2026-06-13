use chrono::{DateTime, Utc};
use poem_openapi::{Enum, Object};
use uuid::Uuid;

use crate::UserResponse;

#[derive(Debug, Enum)]
pub enum EventTypeResponse {
    Push,
    Tag,
    Release,
    PullRequest,
    Cron,
    Manual,
}

#[derive(Debug, Enum)]
pub enum PipelineStatusResponse {
    Evaluating,
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Object)]
#[oai(rename_all = "camelCase")]
pub struct PipelineResponse {
    pub id: Uuid,
    pub serial: i32,
    pub repo_id: Uuid,
    pub title: Option<String>,
    pub triggered_by: String,
    pub triggered_by_user: Option<UserResponse>,
    pub event_type: EventTypeResponse,
    pub event_payload: serde_json::Value,
    pub git_ref: Option<String>,
    pub git_commit_id: String,
    pub status: PipelineStatusResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
