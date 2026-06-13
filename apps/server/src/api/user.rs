use crate::api::security::ApiKeyAuth;

use super::ApiTags;
use api_types::{ForgeInfoResponse, UserEndpointResponse, UserForgeResponse, UserResponse};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::PgPool;
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
        let user = sqlx::query!(r#"SELECT * FROM "user" WHERE id = $1"#, user_id)
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

    /// 나에게 연결된 포지 불러오기
    #[oai(path = "/me/forges", method = "get")]
    async fn get_my_forges(
        &self,
        ApiKeyAuth(user_id): ApiKeyAuth,
        Data(db): Data<&PgPool>,
    ) -> poem::Result<Json<Vec<UserForgeResponse>>> {
        self._get_my_forges(user_id, db).await.map_err(Into::into)
    }

    #[instrument(skip(self, db), err(Debug))]
    async fn _get_my_forges(
        &self,
        user_id: Uuid,
        db: &PgPool,
    ) -> crate::Result<Json<Vec<UserForgeResponse>>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                uf.id as uf_id,
                uf.forge_user_id as uf_forge_user_id,
                uf.created_at as uf_created_at,
                uf.updated_at as uf_updated_at,

                f.id as f_id,
                f.name as f_name
            FROM user_forge uf
            LEFT JOIN forge f ON f.id = uf.forge_id
            WHERE uf.user_id = $1
            ORDER BY uf.created_at
            "#,
            user_id,
        )
        .fetch_all(db)
        .await?;

        Ok(Json(
            rows.into_iter()
                .map(|x| UserForgeResponse {
                    id: x.uf_id,
                    forge_user_id: x.uf_forge_user_id,
                    created_at: x.uf_created_at,
                    updated_at: x.uf_updated_at,

                    forge: ForgeInfoResponse {
                        id: x.f_id,
                        name: x.f_name,
                    },
                })
                .collect(),
        ))
    }
}
