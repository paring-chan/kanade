use std::sync::Arc;

use fred::interfaces::StreamsInterface;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use uuid::Uuid;

use crate::{LogEntry, realtime::Realtime};

pub struct JobsApi;

#[OpenApi(prefix_path = "/jobs", tag = "super::ApiTags::Pipeline")]
impl JobsApi {
    #[oai(path = "/:job_id/logs", method = "get")]
    async fn job_logs(
        &self,
        Path(job_id): Path<Uuid>,
        Data(realtime): Data<&Arc<Realtime>>,
    ) -> poem::Result<Json<Vec<LogEntry>>> {
        self._job_logs(job_id, realtime)
            .await
            .map_err(Into::into)
            .map(Json)
    }

    #[instrument(skip(self, realtime), err(Debug))]
    async fn _job_logs(&self, job_id: Uuid, realtime: &Realtime) -> crate::Result<Vec<LogEntry>> {
        let data: Vec<(String, Vec<(String, String)>)> = realtime
            .client
            .xrange(format!("kanade:logs:{job_id}"), "-", "+", None)
            .await?;

        let logs = data
            .iter()
            .filter_map(|x| x.1.first())
            .filter_map(|x| serde_json::from_str::<LogEntry>(&x.1).ok())
            .collect::<Vec<_>>();

        Ok(logs)
    }
}
