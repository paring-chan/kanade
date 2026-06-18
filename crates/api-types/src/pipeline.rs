use chrono::{DateTime, Utc};
use poem_openapi::{payload::Json, ApiResponse, Enum, Object};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::{ErrorResponse, UserResponse};

#[derive(Debug, Enum)]
#[oai(rename_all = "snake_case")]
pub enum ApiEventType {
    Push,
    Tag,
    Release,
    PullRequest,
    Cron,
    Manual,
}

#[derive(Debug, Clone, Copy, Enum, Serialize, Deserialize, Type)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum PipelineStatusResponse {
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
    pub event_type: ApiEventType,
    pub event_payload: serde_json::Value,
    pub git_ref: Option<String>,
    pub git_commit_id: String,
    pub status: PipelineStatusResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(ApiResponse)]
pub enum GetPipelineResponse {
    #[oai(status = 200)]
    Ok(Json<PipelineResponse>),
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

#[derive(Debug, Enum, Type, Serialize, Deserialize, Clone, Copy)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum JobStatusResponse {
    Waiting,
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
}

#[derive(Debug, Object)]
pub struct PipelineJobResponse {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub timeout: i32,
    pub status: JobStatusResponse,

    pub steps: Vec<PipelineJobStepResponse>,
    pub parents: Vec<Uuid>,

    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Object)]
pub struct PipelineJobStepResponse {
    pub id: Uuid,
    pub name: String,
    pub ordering: i32,
    pub command: String,

    pub created_at: DateTime<Utc>,
    pub exit_code: Option<i32>,

    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}
