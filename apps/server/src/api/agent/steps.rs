use crate::api::ApiTags;
use crate::data::db::JobStatus;
use api_types::{StepFinishRequest, StepFinishResponse, StepStartedResponse};
use poem::web::Data;
use poem_openapi::OpenApi;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct AgentJobStepsApi;

#[OpenApi(prefix_path = "/agent/steps", tag = "ApiTags::Agent")]
impl AgentJobStepsApi {
    /// Job Step 시작 알림
    #[oai(path = "/:id/started", method = "post")]
    async fn started(
        &self,
        db: Data<&PgPool>,
        id: Path<Uuid>,
    ) -> poem::Result<StepStartedResponse> {
        self._started(&db, id.0).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _started(&self, db: &PgPool, step_id: Uuid) -> crate::Result<StepStartedResponse> {
        let mut tx = db.begin().await?;

        let result = sqlx::query!(
            r#"
            UPDATE pipeline_job_step
            SET status = 'running'::job_status,
                started_at = NOW()
            WHERE id = $1 AND status = 'pending'::job_status
            "#,
            step_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(StepStartedResponse::StepNotFound);
        }

        tx.commit().await?;
        Ok(StepStartedResponse::Ok)
    }

    /// Job Step 완료 알림
    #[oai(path = "/:id/finish", method = "post")]
    async fn finish(
        &self,
        db: Data<&PgPool>,
        id: Path<Uuid>,
        request: Json<StepFinishRequest>,
    ) -> poem::Result<StepFinishResponse> {
        self._finish(&db, id.0, request.0).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _finish(
        &self,
        db: &PgPool,
        step_run_id: Uuid,
        request: StepFinishRequest,
    ) -> crate::Result<StepFinishResponse> {
        let mut tx = db.begin().await?;

        let target_status = if request.success {
            JobStatus::Success
        } else {
            JobStatus::Failed
        };

        let result = sqlx::query!(
            r#"
            UPDATE pipeline_job_step
            SET status = $1,
                exit_code = $2,
                finished_at = NOW()
            WHERE id = $3 AND status = 'running'::job_status
            "#,
            target_status as JobStatus,
            request.exit_code,
            step_run_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(StepFinishResponse::StepNotFound);
        }

        tx.commit().await?;
        Ok(StepFinishResponse::Ok)
    }
}
