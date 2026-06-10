use std::sync::Arc;

use crate::{api::security::ApiKeyAuth, forges::AllForges};

use super::ApiTags;
use api_types::{
    ForgeInfoEndpointResponse, ForgeInfoResponse, ForgeRepoResponse, ForgeRepoSearchRequest,
};
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};
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

    /// 포지에서 저장소 검색하기
    #[oai(path = "/:forge_id/search", method = "post")]
    async fn search_repositories(
        &self,
        Path(forge_id): Path<Uuid>,
        Json(payload): Json<ForgeRepoSearchRequest>,
        Data(forges): Data<&Arc<AllForges>>,
        ApiKeyAuth(user_id): ApiKeyAuth,
    ) -> poem::Result<Json<Vec<ForgeRepoResponse>>> {
        let repos = forges
            .search_repositories(user_id, forge_id, &payload.query)
            .await?;

        Ok(Json(
            repos
                .into_iter()
                .map(|x| ForgeRepoResponse {
                    id: x.id,
                    name: x.name,
                })
                .collect(),
        ))
    }
}
