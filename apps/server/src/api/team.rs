use api_types::{
    ErrorResponse, RepoResponse, TeamCreateEndpointResponse, TeamCreateRequest,
    TeamFindOneResponse, TeamResponse,
};
use chrono::{DateTime, Utc};
use garde::Validate;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{api::security::ApiKeyAuth, data::db::RoleType};

pub struct TeamApi;

#[OpenApi(prefix_path = "/teams", tag = "super::ApiTags::Team")]
impl TeamApi {
    #[oai(path = "/", method = "get")]
    async fn list_teams(
        &self,
        ApiKeyAuth(user_id): ApiKeyAuth,
        db: Data<&PgPool>,
    ) -> poem::Result<Json<Vec<TeamResponse>>> {
        self._list_teams(user_id, &db).await.map_err(Into::into)
    }

    async fn _list_teams(
        &self,
        user_id: Uuid,
        db: &PgPool,
    ) -> crate::Result<Json<Vec<TeamResponse>>> {
        #[derive(FromRow)]
        struct TeamRow {
            id: Uuid,
            name: String,
            slug: String,

            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let teams = sqlx::query_as::<_, TeamRow>(
            r#"
            SELECT t.* FROM user_team ut
            LEFT JOIN team t ON t.id = ut.team_id
            WHERE ut.user_id  = $1
            ORDER BY ut.updated_at
            "#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(Json(
            teams
                .into_iter()
                .map(|x| TeamResponse {
                    id: x.id,
                    name: x.name,
                    slug: x.slug,
                    created_at: x.created_at,
                    updated_at: x.updated_at,
                })
                .collect(),
        ))
    }

    #[oai(path = "/:team_slug", method = "get")]
    async fn get_team_by_slug(
        &self,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Path(team_slug): Path<String>,
        db: Data<&PgPool>,
    ) -> poem::Result<TeamFindOneResponse> {
        self._get_team_by_slug(user_id, team_slug, &db)
            .await
            .map_err(Into::into)
    }

    async fn _get_team_by_slug(
        &self,
        user_id: Uuid,
        team_slug: String,
        db: &PgPool,
    ) -> crate::Result<TeamFindOneResponse> {
        #[derive(FromRow)]
        struct TeamRow {
            id: Uuid,
            name: String,
            slug: String,

            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let team = sqlx::query_as::<_, TeamRow>(
            r#"
            SELECT t.* FROM user_team ut
            LEFT JOIN team t ON t.id = ut.team_id
            WHERE ut.user_id  = $1 AND t.slug = $2
            ORDER BY ut.updated_at
            "#,
        )
        .bind(user_id)
        .bind(team_slug)
        .fetch_optional(db)
        .await?;

        match team {
            Some(x) => Ok(TeamFindOneResponse::Ok(Json(TeamResponse {
                id: x.id,
                name: x.name,
                slug: x.slug,
                created_at: x.created_at,
                updated_at: x.updated_at,
            }))),
            None => Ok(TeamFindOneResponse::NotFound),
        }
    }

    #[oai(path = "/:team_slug/repos", method = "get")]
    async fn get_team_repos(
        &self,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Path(team_slug): Path<String>,
        db: Data<&PgPool>,
    ) -> poem::Result<Json<Vec<RepoResponse>>> {
        self._get_team_repos(user_id, team_slug, &db)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _get_team_repos(
        &self,
        user_id: Uuid,
        team_slug: String,
        db: &PgPool,
    ) -> crate::Result<Json<Vec<RepoResponse>>> {
        #[derive(FromRow)]
        struct RepoRow {
            r_id: Uuid,
            r_name: String,
            r_slug: String,
            r_created_at: DateTime<Utc>,
            r_updated_at: DateTime<Utc>,

            t_id: Uuid,
            t_name: String,
            t_slug: String,
            t_created_at: DateTime<Utc>,
            t_updated_at: DateTime<Utc>,
        }

        let res = sqlx::query_as::<_, RepoRow>(
            r#"
            SELECT
                r.id as r_id,
                r.name as r_name,
                r.slug as r_slug,
                r.created_at as r_created_at,
                r.updated_at as r_updated_at,

                t.id as t_id,
                t.name as t_name,
                t.slug as t_slug,
                t.created_at as t_created_at,
                t.updated_at as t_updated_at
            FROM user_team ut
            INNER JOIN team t ON t.id = ut.team_id
            INNER JOIN repo r ON t.id = r.team_id
            WHERE
                ut.user_id = $1 AND
                t.slug = $2
            "#,
        )
        .bind(user_id)
        .bind(team_slug)
        .fetch_all(db)
        .await?;

        Ok(Json(
            res.into_iter()
                .map(|x| RepoResponse {
                    id: x.r_id,
                    name: x.r_name,
                    slug: x.r_slug,
                    created_at: x.r_created_at,
                    updated_at: x.r_updated_at,
                    team: TeamResponse {
                        id: x.t_id,
                        name: x.t_name,
                        slug: x.t_slug,
                        created_at: x.t_created_at,
                        updated_at: x.t_updated_at,
                    },
                })
                .collect(),
        ))
    }

    #[oai(path = "/", method = "post")]
    async fn create_team(
        &self,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Json(payload): Json<TeamCreateRequest>,
        db: Data<&PgPool>,
    ) -> poem::Result<TeamCreateEndpointResponse> {
        self._create_team(payload, user_id, &db)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip(self, payload, user_id, db), err(Debug))]
    async fn _create_team(
        &self,
        payload: TeamCreateRequest,
        user_id: Uuid,
        db: &PgPool,
    ) -> crate::Result<TeamCreateEndpointResponse> {
        match payload.validate() {
            Ok(_) => {}
            Err(report) => {
                return Ok(TeamCreateEndpointResponse::ValidationFailed(Json(
                    serde_json::to_value(report)?,
                )));
            }
        }

        let mut tx = db.begin().await?;

        #[derive(FromRow)]
        struct TeamRow {
            id: Uuid,
            name: String,
            slug: String,

            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let id = Uuid::new_v4();

        let team_result = sqlx::query_as::<_, TeamRow>(
            r#"
            INSERT INTO team
                (id, name, slug)
            VALUES
                ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(payload.name)
        .bind(payload.slug)
        .fetch_one(&mut *tx)
        .await;
        let team_result = match team_result {
            Ok(res) => res,
            Err(e)
                if e.as_database_error()
                    .map(|x| x.is_unique_violation())
                    .unwrap_or_default() =>
            {
                return Ok(TeamCreateEndpointResponse::Conflict(Json(ErrorResponse {
                    message: "팀 슬러그가 이미 사용되고 있습니다".to_string(),
                })));
            }
            Err(e) => return Err(e.into()),
        };

        sqlx::query(
            r#"
            INSERT INTO user_team
                (id, user_id, team_id, role)
            VALUES
                ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(team_result.id)
        .bind(RoleType::Admin)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(TeamCreateEndpointResponse::Ok(Json(
            api_types::TeamResponse {
                id: team_result.id,
                name: team_result.name,
                slug: team_result.slug,

                created_at: team_result.created_at,
                updated_at: team_result.updated_at,
            },
        )))
    }
}
