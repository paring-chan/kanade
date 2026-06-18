use api_types::{AgentStatusResponse, ApiEventType, JobStatusResponse, PipelineStatusResponse};
use sqlx::prelude::Type;

#[derive(Debug, Type, Clone, PartialEq, Eq)]
#[sqlx(type_name = "event_type", rename_all = "snake_case")]
pub enum EventType {
    Push,
    Tag,
    Release,
    PullRequest,
    Cron,
    Manual,
}

impl From<EventType> for ApiEventType {
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

impl From<ApiEventType> for EventType {
    fn from(value: ApiEventType) -> Self {
        match value {
            ApiEventType::Push => Self::Push,
            ApiEventType::Tag => Self::Tag,
            ApiEventType::Release => Self::Release,
            ApiEventType::PullRequest => Self::PullRequest,
            ApiEventType::Cron => Self::Cron,
            ApiEventType::Manual => Self::Manual,
        }
    }
}

#[derive(Debug, Type, Clone, Copy)]
#[sqlx(type_name = "pipeline_status", rename_all = "snake_case")]
pub enum PipelineStatus {
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
}

impl From<PipelineStatus> for PipelineStatusResponse {
    fn from(value: PipelineStatus) -> Self {
        match value {
            PipelineStatus::Queued => Self::Queued,
            PipelineStatus::Running => Self::Running,
            PipelineStatus::Success => Self::Success,
            PipelineStatus::Failed => Self::Failed,
            PipelineStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Type, Clone, Copy)]
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

impl From<JobStatus> for JobStatusResponse {
    fn from(value: JobStatus) -> Self {
        match value {
            JobStatus::Waiting => Self::Waiting,
            JobStatus::Pending => Self::Pending,
            JobStatus::Running => Self::Running,
            JobStatus::Success => Self::Success,
            JobStatus::Failed => Self::Failed,
            JobStatus::Skipped => Self::Skipped,
            JobStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Type)]
#[sqlx(type_name = "role_type", rename_all = "snake_case")]
pub enum RoleType {
    Viewer,
    Manager,
    Admin,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "agent_status", rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
}

impl From<AgentStatus> for AgentStatusResponse {
    fn from(value: AgentStatus) -> Self {
        match value {
            AgentStatus::Idle => AgentStatusResponse::Idle,
            AgentStatus::Busy => Self::Busy,
            AgentStatus::Offline => Self::Offline,
        }
    }
}
