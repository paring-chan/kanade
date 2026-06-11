use std::sync::Arc;

use oauth2::{RefreshToken, TokenResponse};
use reqwest::header::AUTHORIZATION;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::{
    config::AppConfig,
    data::forges::ForgejoForgeConfig,
    error::AppError,
    forges::UpstreamRepositoryInfo,
    util::{HTTP, OAUTH2_REQWEST},
};

pub struct ForgejoApi {
    config: Arc<AppConfig>,
}

#[derive(Deserialize, Debug)]
struct RepoSearchResults {
    ok: bool,
    data: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    id: i64,
    full_name: String,
    permissions: RepoPermissions,
}

#[derive(Debug, Deserialize)]
struct RepoPermissions {
    admin: bool,
}

impl ForgejoApi {
    pub fn new(config: Arc<AppConfig>) -> crate::Result<Self> {
        Ok(Self { config })
    }

    pub async fn search_repositories(
        &self,
        config: &ForgejoForgeConfig,
        uid: &str,
        access_token: &SecretString,
        search: &str,
    ) -> crate::Result<Vec<super::UpstreamRepositoryInfo>> {
        let res = HTTP
            .get(format!(
                "{}/api/v1/repos/search?q={}&uid={}&sort=alpha&limit=30",
                &config.url,
                urlencoding::encode(search),
                urlencoding::encode(uid),
            ))
            .header(
                AUTHORIZATION,
                format!("token {}", access_token.expose_secret()),
            )
            .send()
            .await?
            .error_for_status()?
            .json::<RepoSearchResults>()
            .await?;

        Ok(res
            .data
            .into_iter()
            .filter(|x| x.permissions.admin)
            .map(|x| UpstreamRepositoryInfo {
                id: x.id.to_string(),
                name: x.full_name,
            })
            .collect())
    }

    pub async fn refresh_token(
        &self,
        config: &ForgejoForgeConfig,
        refresh_token: RefreshToken,
    ) -> crate::Result<impl TokenResponse> {
        let client = config.oauth2_client(&self.config)?;

        let response = client
            .exchange_refresh_token(&refresh_token)
            .request_async(&*OAUTH2_REQWEST)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(response)
    }
}
