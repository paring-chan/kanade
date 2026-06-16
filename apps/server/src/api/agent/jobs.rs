use std::{collections::HashMap, sync::Arc};

use crate::EventMessage;
use crate::data::db::JobStatus;
use crate::{api::ApiTags, realtime::Realtime};
use api_types::{
    AgentPipelineJobResponse, AgentPipelineJobStepResponse, JobAcquireEndpointResponse,
    JobAcquireResponse, JobFinishRequest, JobFinishResponse,
};
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sqlx::PgPool;
use uuid::Uuid;

pub struct AgentJobsApi;

#[OpenApi(prefix_path = "/agent/jobs", tag = "ApiTags::Agent")]
impl AgentJobsApi {
    /// Job 획득
    #[oai(path = "/acquire", method = "post")]
    async fn acquire(
        &self,
        Data(db): Data<&PgPool>,
        Data(realtime): Data<&Arc<Realtime>>,
    ) -> poem::Result<JobAcquireEndpointResponse> {
        self._acquire(db, realtime).await.map_err(Into::into)
    }

    #[instrument(skip(self, db, realtime), err(Debug))]
    async fn _acquire(
        &self,
        db: &PgPool,
        realtime: &Realtime,
    ) -> crate::Result<JobAcquireEndpointResponse> {
        let mut tx = db.begin().await?;

        let row = sqlx::query!(
            r#"
            SELECT j.id, j.name, j.timeout, j.image, j.pipeline_id
            FROM pipeline_job j
            WHERE j.status = 'pending'::job_status
            ORDER BY j.created_at ASC
            LIMIT 1
            FOR UPDATE OF j SKIP LOCKED
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(job) = row else {
            tx.rollback().await?;
            return Ok(JobAcquireEndpointResponse::NoContent);
        };

        sqlx::query!(
            r#"
            UPDATE pipeline_job
            SET status = 'running'::job_status,
                started_at = NOW()
            WHERE id = $1
            "#,
            job.id
        )
        .execute(&mut *tx)
        .await?;

        let job_res = AgentPipelineJobResponse {
            id: job.id,
            name: job.name,
            timeout: job.timeout,
            image: job.image,
        };

        let steps = sqlx::query!(
            r#"
                SELECT
                    s.id,
                    s.name,
                    s.ordering,
                    s.command
                FROM pipeline_job_step s
                WHERE
                    s.job_id = $1
            "#,
            job_res.id
        )
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .map(|x| AgentPipelineJobStepResponse {
            id: x.id,
            name: x.name,
            ordering: x.ordering,
            command: x.command,
        })
        .collect::<Vec<_>>();

        let run = JobAcquireResponse {
            id: job_res.id,
            job: job_res,
            steps,
            env: HashMap::default(),
            secrets: HashMap::default(),
        };

        tx.commit().await?;

        realtime
            .publish(&EventMessage::UpdateJobStatus {
                pipeline: job.pipeline_id,
                job: run.id,
                status: api_types::JobStatusResponse::Running,
            })
            .await?;

        Ok(JobAcquireEndpointResponse::Ok(Json(run)))
    }

    /// Job 완료
    #[oai(path = "/:id/finish", method = "post")]
    async fn finish(
        &self,
        Data(db): Data<&PgPool>,
        Data(realtime): Data<&Arc<Realtime>>,
        id: Path<Uuid>,
        status: Json<JobFinishRequest>,
    ) -> poem::Result<JobFinishResponse> {
        self._finish(db, realtime, id.0, status.0)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip(self, realtime, db), err(Debug))]
    async fn _finish(
        &self,
        db: &PgPool,
        realtime: &Realtime,
        run_id: Uuid,
        request: JobFinishRequest,
    ) -> crate::Result<JobFinishResponse> {
        let mut tx = db.begin().await?;

        let job_status = if request.success {
            JobStatus::Success
        } else {
            JobStatus::Failed
        };

        let result = sqlx::query!(
            r#"
            UPDATE pipeline_job j
            SET status = $1,
                finished_at = NOW()
            WHERE id = $2 AND status = 'running'::job_status
            RETURNING j.id, j.pipeline_id
            "#,
            job_status as JobStatus,
            run_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(result) = result else {
            tx.rollback().await?;
            return Ok(JobFinishResponse::JobNotFound);
        };

        let step_status = if request.success {
            JobStatus::Success
        } else {
            JobStatus::Skipped
        };

        let _affected_steps = sqlx::query!(
            r#"
            UPDATE pipeline_job_step s
            SET status = CASE
                WHEN status = 'running'::job_status THEN 'failed'::job_status
                ELSE $1
            END,
                finished_at = NOW()
            WHERE job_id = $2 AND status NOT IN ('success', 'failed', 'skipped', 'cancelled')
            RETURNING s.id
            "#,
            step_status as JobStatus,
            run_id
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        realtime
            .publish(&EventMessage::UpdateJobStatus {
                pipeline: result.pipeline_id,
                job: result.id,
                status: job_status.into(),
            })
            .await?;

        // TODO: publish step status update

        Ok(JobFinishResponse::Ok)
    }
}
