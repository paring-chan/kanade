use super::ApiTags;
use api_types::{ForgeInfoEndpointResponse, ForgeInfoResponse};
use poem::web::Data;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

pub struct ForgeApi;

#[OpenApi(prefix_path = "/forges", tag = "ApiTags::Forge")]
impl ForgeApi {
    /// 포지 목록
    #[oai(path = "/", method = "get")]
    async fn list_forges(&self, db: Data<&PgPool>) -> poem::Result<ForgeInfoEndpointResponse> {
        self._list_forges(&db).await.map_err(Into::into)
    }

    #[instrument(skip(self, db))]
    async fn _list_forges(&self, db: &PgPool) -> crate::Result<ForgeInfoEndpointResponse> {
        #[derive(FromRow, Debug)]
        struct ForgeRow {
            id: Uuid,
            name: String,
        }

        let forges = sqlx::query_as::<_, ForgeRow>(
            r#"
            SELECT id, name, config
            FROM forge
            ORDER BY created_at
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(ForgeInfoEndpointResponse::Ok(Json(
            forges
                .into_iter()
                .map(|x| ForgeInfoResponse {
                    id: x.id,
                    name: x.name,
                })
                .collect(),
        )))
    }
}
