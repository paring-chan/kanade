use crate::api::security::ApiKeyAuth;

use super::ApiTags;
use api_types::{UserEndpointResponse, UserResponse};
use chrono::{DateTime, Utc};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

pub struct UserApi;

#[OpenApi(prefix_path = "/users", tag = "ApiTags::User")]
impl UserApi {
    /// 내 정보 불러오기
    #[oai(path = "/me", method = "get")]
    async fn get_me(
        &self,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Data(db): Data<&PgPool>,
    ) -> poem::Result<UserEndpointResponse> {
        self._get_me(user_id, db).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _get_me(&self, user_id: Uuid, db: &PgPool) -> crate::Result<UserEndpointResponse> {
        #[derive(FromRow)]
        struct UserRow {
            id: Uuid,
            username: String,
            nick: Option<String>,
            email: Option<String>,
            avatar_url: Option<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let user = sqlx::query_as::<_, UserRow>(r#"SELECT * FROM "user" WHERE id = $1"#)
            .bind(user_id)
            .fetch_one(db)
            .await?;

        Ok(UserEndpointResponse::Ok(Json(UserResponse {
            id: user.id,
            username: user.username,
            nick: user.nick,
            email: user.email,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })))
    }
}
