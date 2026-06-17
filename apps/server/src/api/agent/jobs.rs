use std::{collections::HashMap, sync::Arc};

use crate::EventMessage;
use crate::api::security::AgentTokenAuth;
use crate::crypto::CryptoEngine;
use crate::data::db::{JobStatus, PipelineStatus};
use crate::security::DatabaseSecurityExt;
use crate::{api::ApiTags, realtime::Realtime};
use api_types::{
    AgentPipelineJobResponse, AgentPipelineJobStepResponse, JobAcquireEndpointResponse,
    JobAcquireResponse, JobFinishRequest, JobFinishResponse, PipelineStatusResponse,
};
use event_evaluator::types::EnvDefinition;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use secrecy::ExposeSecret;
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
        Data(crypto): Data<&Arc<CryptoEngine>>,
        AgentTokenAuth(agent_id): AgentTokenAuth,
    ) -> poem::Result<JobAcquireEndpointResponse> {
        self._acquire(db, realtime, crypto, agent_id)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip(self, db, realtime, crypto), err(Debug))]
    async fn _acquire(
        &self,
        db: &PgPool,
        realtime: &Realtime,
        crypto: &CryptoEngine,
        agent_id: Uuid,
    ) -> crate::Result<JobAcquireEndpointResponse> {
        let mut tx = db.begin().await?;

        let row = sqlx::query!(
            r#"
            SELECT
                j.id,
                j.name,
                j.timeout,
                j.image,
                j.pipeline_id,
                j.env as "env: sqlx::types::Json<HashMap<String, EnvDefinition>>",
                r.ssh_key
            FROM pipeline_job j
            INNER JOIN pipeline p ON p.id = j.pipeline_id
            INNER JOIN repo r ON r.id = p.repo_id
            WHERE j.status = 'pending'::job_status
                AND NOT EXISTS (
                    SELECT
                    FROM pipeline_job_depend d
                    INNER JOIN pipeline_job upstream ON upstream.id = d.upstream_id
                    WHERE d.downstream_id = j.id
                    AND upstream.status != 'success'::job_status
                )
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

        let ssh_key = crypto.decrypt(&job.ssh_key)?;

        sqlx::query!(
            r#"
            SELECT FROM pipeline
            WHERE id = $1
            FOR UPDATE
            "#,
            job.pipeline_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            UPDATE pipeline_job
            SET status = 'running'::job_status,
                started_at = NOW(),
                agent_id = $2
            WHERE id = $1
            "#,
            job.id,
            agent_id
        )
        .execute(&mut *tx)
        .await?;

        let pipeline_started = sqlx::query!(
            r#"
            UPDATE pipeline
            SET
                status = 'running'::pipeline_status,
                started_at = now()
            WHERE
                id = $1
                AND status = 'queued'::pipeline_status
                AND started_at IS NULL
            RETURNING id
            "#,
            job.pipeline_id,
        )
        .fetch_optional(&mut *tx)
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
                    s.command,
                    s.env as "env: sqlx::types::Json<HashMap<String, EnvDefinition>>"
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
            env: x.env.0.into_iter().map(|(k, v)| (k, v.into())).collect(),
        })
        .collect::<Vec<_>>();

        let run = JobAcquireResponse {
            id: job_res.id,
            job: job_res,
            steps,
            env: job.env.0.into_iter().map(|(k, v)| (k, v.into())).collect(),
            secrets: HashMap::default(),
            ssh_key: ssh_key.expose_secret().to_string(),
        };

        tx.commit().await?;

        if pipeline_started.is_some() {
            realtime
                .publish(&EventMessage::UpdatePipelineStatus {
                    pipeline: job.pipeline_id,
                    status: PipelineStatusResponse::Running,
                })
                .await?;
        }

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

        sqlx::query!(
            r#"
            SELECT FROM pipeline
            WHERE id = $1
            FOR UPDATE
            "#,
            result.pipeline_id
        )
        .execute(&mut *tx)
        .await?;

        let updated_pipeline = sqlx::query!(
            r#"
            UPDATE pipeline
            SET status = CASE
                WHEN EXISTS(
                    SELECT 1 FROM pipeline_job
                    WHERE pipeline_id = $1 AND status IN ('failed'::job_status, 'cancelled'::job_status)
                ) THEN 'failed'::pipeline_status
                ELSE 'success'::pipeline_status
                END,
                finished_at = now()
            WHERE
                id = $1 AND status = 'running'::pipeline_status
                AND NOT EXISTS (
                    SELECT 1 FROM pipeline_job
                    WHERE pipeline_id = $1 AND finished_at IS NULL
                )
            RETURNING id, status as "status: PipelineStatus"
            "#,
            result.pipeline_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        realtime
            .publish(&EventMessage::UpdateJobStatus {
                pipeline: result.pipeline_id,
                job: result.id,
                status: job_status.into(),
            })
            .await?;

        if let Some(pipeline) = updated_pipeline {
            realtime
                .publish(&EventMessage::UpdatePipelineStatus {
                    pipeline: pipeline.id,
                    status: pipeline.status.into(),
                })
                .await?;
        }

        // TODO: publish step status update

        Ok(JobFinishResponse::Ok)
    }
}
