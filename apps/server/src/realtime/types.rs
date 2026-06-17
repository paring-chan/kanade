use api_types::{JobStatusResponse, PipelineStatusResponse};
use poem_openapi::Object;
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

#[derive(Debug, Clone, Serialize, Deserialize, Object, specta::Type)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct LogEntry {
    pub step_id: Uuid,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LogMessage {
    pub job_id: Uuid,
    pub entry: LogEntry,
}
