use std::collections::HashMap;

use crate::api::ApiTags;
use crate::data::db::JobStatus;
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
    async fn acquire(&self, db: Data<&PgPool>) -> poem::Result<JobAcquireEndpointResponse> {
        self._acquire(&db).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _acquire(&self, db: &PgPool) -> crate::Result<JobAcquireEndpointResponse> {
        let mut tx = db.begin().await?;

        let row = sqlx::query!(
            r#"
            SELECT j.id, j.name, j.timeout, j.image
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

        let job = AgentPipelineJobResponse {
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
            job.id
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
            id: job.id,
            job,
            steps,
            env: HashMap::default(),
            secrets: HashMap::default(),
        };

        tx.commit().await?;
        Ok(JobAcquireEndpointResponse::Ok(Json(run)))
    }

    /// Job 완료
    #[oai(path = "/:id/finish", method = "post")]
    async fn finish(
        &self,
        db: Data<&PgPool>,
        id: Path<Uuid>,
        status: Json<JobFinishRequest>,
    ) -> poem::Result<JobFinishResponse> {
        self._finish(&db, id.0, status.0).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _finish(
        &self,
        db: &PgPool,
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
            UPDATE pipeline_job
            SET status = $1,
                finished_at = NOW()
            WHERE id = $2 AND status = 'running'::job_status
            "#,
            job_status as JobStatus,
            run_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(JobFinishResponse::JobNotFound);
        }

        let step_status = if request.success {
            JobStatus::Success
        } else {
            JobStatus::Skipped
        };

        sqlx::query!(
            r#"
            UPDATE pipeline_job_step
            SET status = CASE
                WHEN status = 'running'::job_status THEN 'failed'::job_status
                ELSE $1
            END,
                finished_at = NOW()
            WHERE job_id = $2 AND status NOT IN ('success', 'failed', 'skipped', 'cancelled')
            "#,
            step_status as JobStatus,
            run_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(JobFinishResponse::Ok)
    }
}
