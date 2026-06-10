use api_types::{ErrorResponse, TeamCreateEndpointResponse, TeamCreateRequest};
use chrono::{DateTime, Utc};
use garde::Validate;
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{api::security::ApiKeyAuth, data::db::RoleType};

pub struct TeamApi;

#[OpenApi(prefix_path = "/teams", tag = "super::ApiTags::Team")]
impl TeamApi {
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
