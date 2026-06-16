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
