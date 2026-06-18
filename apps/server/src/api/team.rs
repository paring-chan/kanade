use std::sync::Arc;

use api_types::{
    ErrorResponse, RepoResponse, SecretCreateEndpointResponse, SecretCreateRequest, SecretInfo,
    TeamCreateEndpointResponse, TeamCreateRequest, TeamFindOneResponse, TeamResponse,
};
use garde::Validate;
use itertools::Itertools;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::security::ApiKeyAuth,
    crypto::CryptoEngine,
    data::db::{EventType, RoleType},
    security::DatabaseSecurityExt,
};

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
        let mut tx = db.begin_as(user_id).await?;

        let teams = sqlx::query!(
            r#"
            SELECT t.* FROM user_team ut
            LEFT JOIN team t ON t.id = ut.team_id
            WHERE ut.user_id  = $1
            ORDER BY ut.updated_at
            "#,
            user_id
        )
        .fetch_all(&mut *tx)
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
        let mut tx = db.begin_as(user_id).await?;

        let team = sqlx::query!(
            r#"
            SELECT t.* FROM user_team ut
            LEFT JOIN team t ON t.id = ut.team_id
            WHERE ut.user_id  = $1 AND t.slug = $2
            ORDER BY ut.updated_at
            "#,
            user_id,
            team_slug,
        )
        .fetch_optional(&mut *tx)
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
        let mut tx = db.begin_as(user_id).await?;

        let res = sqlx::query!(
            r#"
            SELECT
                r.id as r_id,
                r.name as r_name,
                r.slug as r_slug,
                r.repo_url as r_repo_url,
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
            user_id,
            team_slug,
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(Json(
            res.into_iter()
                .map(|x| RepoResponse {
                    id: x.r_id,
                    name: x.r_name,
                    slug: x.r_slug,
                    upstream_url: x.r_repo_url,
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

        let mut tx = db.begin_as(user_id).await?;

        let id = Uuid::new_v4();

        let team_result = sqlx::query!(
            r#"
            INSERT INTO team
                (id, name, slug)
            VALUES
                ($1, $2, $3)
            RETURNING id, name, slug, created_at, updated_at
            "#,
            id,
            payload.name,
            payload.slug
        )
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

        sqlx::query!(
            r#"
            INSERT INTO user_team
                (id, user_id, team_id, role)
            VALUES
                ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            user_id,
            team_result.id,
            RoleType::Admin as RoleType
        )
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

    #[oai(path = "/:team_slug/secrets", method = "get")]
    async fn list_secrets(
        &self,
        Data(db): Data<&PgPool>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Path(team_slug): Path<String>,
    ) -> poem::Result<Json<Vec<SecretInfo>>> {
        self._list_secrets(db, user_id, team_slug)
            .await
            .map(Json)
            .map_err(Into::into)
    }

    #[instrument(skip(self), err(Debug))]
    async fn _list_secrets(
        &self,
        db: &PgPool,
        user_id: Uuid,
        team_slug: String,
    ) -> crate::Result<Vec<SecretInfo>> {
        let mut tx = db.begin_as(user_id).await?;

        let secrets = sqlx::query!(
            r#"
            SELECT s.id, s.key, s.created_at, s.updated_at
            FROM team t
            INNER JOIN team_secret ts ON ts.team_id = t.id
            INNER JOIN secret s ON ts.secret_id = s.id
            INNER JOIN user_team ut ON t.id = ut.team_id
            WHERE t.slug = $1 AND ut.user_id = $2
            "#,
            team_slug,
            user_id
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(secrets
            .into_iter()
            .map(|x| SecretInfo {
                id: x.id,
                key: x.key,
                created_at: x.created_at,
                updated_at: x.updated_at,
            })
            .collect())
    }

    #[oai(path = "/:team_slug/secrets", method = "post")]
    async fn create_secret(
        &self,
        Data(db): Data<&PgPool>,
        Data(crypto): Data<&Arc<CryptoEngine>>,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Path(team_slug): Path<String>,
        Json(payload): Json<SecretCreateRequest>,
    ) -> poem::Result<SecretCreateEndpointResponse> {
        self._create_secret(db, crypto, user_id, team_slug, payload)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip(self, crypto, db, payload), err(Debug))]
    async fn _create_secret(
        &self,
        db: &PgPool,
        crypto: &CryptoEngine,
        user_id: Uuid,
        team_slug: String,
        payload: SecretCreateRequest,
    ) -> crate::Result<SecretCreateEndpointResponse> {
        match payload.validate() {
            Ok(_) => {}
            Err(report) => {
                return Ok(SecretCreateEndpointResponse::ValidationFailed(Json(
                    serde_json::to_value(report)?,
                )));
            }
        }

        let mut tx = db.begin_bypass().await?;

        let team = match sqlx::query!(
            r#"
            SELECT t.id
            FROM team t
            INNER JOIN user_team ut ON t.id = ut.team_id
            WHERE ut.user_id = $1 AND ut.role IN ('manager'::role_type, 'admin'::role_type)
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            Some(v) => v,
            None => {
                return Ok(SecretCreateEndpointResponse::NotFound(Json(
                    ErrorResponse {
                        message: "team not found".to_string(),
                    },
                )));
            }
        };

        let secret = sqlx::query!(
            r#"
            INSERT INTO secret
                (id, key, value)
            VALUES
                ($1, $2, $3)
            RETURNING id, key, value, created_at, updated_at
            "#,
            Uuid::new_v4(),
            payload.key,
            crypto.encrypt(&payload.value)?,
        )
        .fetch_one(&mut *tx)
        .await?;

        let scopes = payload
            .scopes
            .into_iter()
            .map(EventType::from)
            .dedup()
            .collect::<Vec<EventType>>();

        sqlx::query!(
            r#"
            INSERT INTO team_secret
                (id, team_id, secret_id, scopes)
            VALUES
                ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            team.id,
            secret.id,
            scopes as Vec<EventType>
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(SecretCreateEndpointResponse::Ok)
    }
}
