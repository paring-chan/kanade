use api_types::{JobStatusResponse, PipelineStatusResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "t", content = "p", rename_all = "camelCase")]
pub enum EventMessage {
    UpdatePipelineStatus {
        pipeline: Uuid,
        status: PipelineStatusResponse,
    },
    UpdateJobStatus {
        pipeline: Uuid,
        job: Uuid,
        status: JobStatusResponse,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum LogKind {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LogEntry {
    pub step_id: Uuid,
    pub content: String,
    pub kind: LogKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "t", content = "p", rename_all = "camelCase")]
pub enum LogMessage {
    Log { job_id: Uuid, entry: LogEntry },
}
