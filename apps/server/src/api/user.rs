use super::ApiTags;
use poem_openapi::OpenApi;

pub struct UserApi;

#[OpenApi(prefix_path = "/users", tag = "ApiTags::User")]
impl UserApi {
    /// 내 정보 불러오기
    #[oai(path = "/me", method = "get")]
    async fn get_me(&self) {}
}
