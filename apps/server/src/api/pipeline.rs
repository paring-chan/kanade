use std::collections::HashMap;

use api_types::{
    ErrorResponse, GetPipelineResponse, PipelineJobResponse, PipelineJobStepResponse,
    PipelineResponse, UserResponse,
};
use itertools::Itertools;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::security::ApiKeyAuth,
    data::db::{EventType, JobStatus, PipelineStatus},
    security::DatabaseSecurityExt,
};

pub struct PipelineApi;

#[OpenApi(prefix_path = "/pipelines", tag = "super::ApiTags::Pipeline")]
impl PipelineApi {
    #[oai(path = "/:pipeline_id", method = "get")]
    async fn get_pipeline(
        &self,
        Data(db): Data<&PgPool>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Path(pipeline_id): Path<Uuid>,
    ) -> poem::Result<GetPipelineResponse> {
        self._get_pipeline(db, user_id, pipeline_id)
            .await
            .map_err(Into::into)
    }

    async fn _get_pipeline(
        &self,
        db: &PgPool,
        user_id: Uuid,
        pipeline_id: Uuid,
    ) -> crate::Result<GetPipelineResponse> {
        let mut tx = db.begin_as(user_id).await?;

        let Some(row) = sqlx::query!(
            r#"
            SELECT
                p.id as p_id,
                p.serial as p_serial,
                p.repo_id as p_repo_id,
                p.title as p_title,
                p.triggered_by as p_triggered_by,
                p.event_type as "event_type: EventType",
                p.event_payload as p_event_payload,
                p.git_ref as p_git_ref,
                p.git_commit_id as p_git_commit_id,
                p.status as "status: PipelineStatus",
                p.created_at as p_created_at,
                p.updated_at as p_updated_at,

                tu.id as "tu_id?",
                tu.username as "tu_username?",
                tu.nick as "tu_nick?",
                tu.email as "tu_email?",
                tu.avatar_url as "tu_avatar_url?",
                tu.created_at as "tu_created_at?",
                tu.updated_at as "tu_updated_at?"
            FROM
                pipeline p
            INNER JOIN repo r ON r.id = p.repo_id
            INNER JOIN team t ON t.id = r.team_id
            LEFT JOIN "user" tu ON tu.id = p.triggered_by_user
            WHERE
                p.id = $1
            "#,
            pipeline_id
        )
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Ok(GetPipelineResponse::NotFound(Json(ErrorResponse {
                message: "파이프라인을 찾을 수 없습니다".to_string(),
            })));
        };

        Ok(GetPipelineResponse::Ok(Json(PipelineResponse {
            id: row.p_id,
            serial: row.p_serial,
            repo_id: row.p_repo_id,
            title: row.p_title,
            triggered_by: row.p_triggered_by,
            triggered_by_user: row.tu_id.map(|id| UserResponse {
                id,
                username: row.tu_username.expect("must exist"),
                nick: row.tu_nick,
                email: row.tu_email,
                avatar_url: row.tu_avatar_url,
                created_at: row.tu_created_at.expect("must exist"),
                updated_at: row.tu_updated_at.expect("must exist"),
            }),
            event_type: row.event_type.into(),
            event_payload: row.p_event_payload,
            git_ref: row.p_git_ref,
            git_commit_id: row.p_git_commit_id,
            status: row.status.into(),
            created_at: row.p_created_at,
            updated_at: row.p_updated_at,
        })))
    }

    #[oai(path = "/:pipeline_id/jobs", method = "get")]
    async fn get_pipeline_jobs(
        &self,
        Data(db): Data<&PgPool>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Path(pipeline_id): Path<Uuid>,
    ) -> poem::Result<Json<Vec<PipelineJobResponse>>> {
        self._get_pipeline_jobs(db, user_id, pipeline_id)
            .await
            .map_err(Into::into)
            .map(Json)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _get_pipeline_jobs(
        &self,
        db: &PgPool,
        user_id: Uuid,
        pipeline_id: Uuid,
    ) -> crate::Result<Vec<PipelineJobResponse>> {
        let mut tx = db.begin_as(user_id).await?;

        let rows = sqlx::query!(
            r#"
            SELECT
                j.id as j_id,
                j.key as j_key,
                j.name as j_name,
                j.timeout as j_timeout,
                j.status as "j_status: JobStatus",
                j.created_at as j_created_at,
                j.started_at as j_started_at,
                j.finished_at as j_finished_at,

                s.id as "s_id?",
                s.name as "s_name?",
                s.ordering as "s_ordering?",
                s.command as "s_command?",
                s.created_at as "s_created_at?",
                s.exit_code as "s_exit_code?",
                s.started_at as "s_started_at?",
                s.finished_at as "s_finished_at?"
            FROM
                pipeline p
            INNER JOIN repo r ON r.id = p.repo_id
            INNER JOIN team t ON t.id = r.team_id
            INNER JOIN pipeline_job j ON j.pipeline_id = p.id
            LEFT JOIN pipeline_job_step s ON s.job_id = j.id
            WHERE
                p.id = $1
            ORDER BY j.id ASC, s.ordering ASC
            "#,
            pipeline_id
        )
        .fetch_all(&mut *tx)
        .await?;

        let depend_rows = sqlx::query!(
            r#"
            SELECT
                d.upstream_id, d.downstream_id
            FROM pipeline_job_depend d
            INNER JOIN pipeline_job j ON j.id = d.downstream_id
            INNER JOIN pipeline p ON p.id = j.pipeline_id
            WHERE p.id = $1
            ORDER BY d.downstream_id ASC
            "#,
            pipeline_id
        )
        .fetch_all(&mut *tx)
        .await?;

        let depends = depend_rows.into_iter().chunk_by(|r| r.downstream_id);
        let depend_map = depends
            .into_iter()
            .map(|(downstream, group)| {
                let upstreams = group.map(|r| r.upstream_id).collect::<Vec<_>>();
                (downstream, upstreams)
            })
            .collect::<HashMap<_, _>>();

        let mut jobs = Vec::<PipelineJobResponse>::new();

        for row in rows.into_iter() {
            let job = match jobs.last_mut().filter(|j| j.id == row.j_id) {
                Some(j) => j,
                None => jobs.push_mut(PipelineJobResponse {
                    id: row.j_id,
                    key: row.j_key,
                    name: row.j_name,
                    timeout: row.j_timeout,
                    steps: vec![],
                    parents: depend_map.get(&row.j_id).cloned().unwrap_or_default(),
                    status: row.j_status.into(),
                    created_at: row.j_created_at,
                    started_at: row.j_started_at,
                    finished_at: row.j_finished_at,
                }),
            };

            if let Some(step_id) = row.s_id
                && job.steps.last().is_none_or(|s| s.id != step_id)
            {
                job.steps.push(PipelineJobStepResponse {
                    id: step_id,
                    name: row.s_name.expect("must exist"),
                    ordering: row.s_ordering.expect("must exist"),
                    command: row.s_command.expect("must exist"),
                    created_at: row.s_created_at.expect("must exist"),
                    exit_code: row.s_exit_code,
                    started_at: row.s_started_at,
                    finished_at: row.s_finished_at,
                });
            }
        }

        Ok(jobs)
    }
}
