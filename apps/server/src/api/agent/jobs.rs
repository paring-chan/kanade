use crate::api::ApiTags;
use api_types::{
    AcquireResponse, PipelineJobResponse, PipelineJobRunResponse, PipelineJobStepResponse,
    PipelineJobStepRunResponse,
};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

pub struct AgentJobsApi;

#[OpenApi(prefix_path = "/agent/jobs", tag = "ApiTags::Agent")]
impl AgentJobsApi {
    /// Job 획득
    #[oai(path = "/acquire", method = "post")]
    async fn acquire(&self, db: Data<&PgPool>) -> poem::Result<AcquireResponse> {
        self._acquire(&db).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _acquire(&self, db: &PgPool) -> crate::Result<AcquireResponse> {
        let mut tx = db.begin().await?;

        #[derive(FromRow)]
        struct JobRow {
            run_id: Uuid,
            run_attempt_serial: i32,

            job_id: Uuid,
            job_name: String,
            job_timeout: i32,
            job_image: String,
        }

        let row = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT job_run.id as run_id, job_run.attempt_serial as run_attempt_serial,
                   job.id as job_id, job.name as job_name, job.timeout as job_timeout, job.image as job_image
            FROM pipeline_job_run job_run
            LEFT JOIN pipeline_job job ON job_run.job_id = job.id
            WHERE job_run.status = 'pending'::job_status
            ORDER BY job_run.created_at ASC
            LIMIT 1
            FOR UPDATE OF job_run SKIP LOCKED
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(AcquireResponse::NoContent);
        };

        sqlx::query(
            r#"
            UPDATE pipeline_job_run
            SET status = 'running'::job_status,
                started_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(row.run_id)
        .execute(&mut *tx)
        .await?;

        let job = PipelineJobResponse {
            id: row.job_id,
            name: row.job_name,
            timeout: row.job_timeout,
            image: row.job_image,
        };

        #[derive(FromRow)]
        struct StepRow {
            run_id: Uuid,

            step_id: Uuid,
            step_name: String,
            step_ordering: i32,
            step_command: String,
        }

        let steps = sqlx::query_as::<_, StepRow>(
            r#"
                SELECT run.id as run_id,
                    -- step
                    step.id as step_id,
                    step.name as step_name,
                    step.ordering as step_ordering,
                    step.command as step_command
                FROM pipeline_job_step_run run
                     LEFT JOIN pipeline_job_step step ON step.id = run.step_id
                WHERE
                    run.run_id = $1
            "#,
        )
        .bind(row.run_id)
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .map(|x| PipelineJobStepRunResponse {
            id: x.run_id,
            step: PipelineJobStepResponse {
                id: x.step_id,
                name: x.step_name,
                ordering: x.step_ordering,
                command: x.step_command,
            },
        })
        .collect::<Vec<_>>();

        let run = PipelineJobRunResponse {
            id: row.run_id,
            attempt_serial: row.run_attempt_serial,
            job,
            steps,
        };

        tx.commit().await?;
        Ok(AcquireResponse::Ok(Json(run)))
    }
}
