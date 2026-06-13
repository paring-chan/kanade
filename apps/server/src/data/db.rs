use api_types::{EventTypeResponse, PipelineStatusResponse};
use sqlx::prelude::Type;

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

impl From<EventType> for EventTypeResponse {
    fn from(value: EventType) -> Self {
        match value {
            EventType::Push => Self::Push,
            EventType::Tag => Self::Tag,
            EventType::Release => Self::Release,
            EventType::PullRequest => Self::PullRequest,
            EventType::Cron => Self::Cron,
            EventType::Manual => Self::Manual,
        }
    }
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

impl From<PipelineStatus> for PipelineStatusResponse {
    fn from(value: PipelineStatus) -> Self {
        match value {
            PipelineStatus::Evaluating => Self::Evaluating,
            PipelineStatus::Queued => Self::Queued,
            PipelineStatus::Running => Self::Running,
            PipelineStatus::Success => Self::Success,
            PipelineStatus::Failed => Self::Failed,
            PipelineStatus::Cancelled => Self::Cancelled,
        }
    }
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

#[derive(Debug, Type)]
#[sqlx(type_name = "role_type", rename_all = "snake_case")]
pub enum RoleType {
    Viewer,
    Manager,
    Admin,
}
