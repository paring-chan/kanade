use std::sync::Arc;

use oauth2::{RefreshToken, TokenResponse};
use reqwest::{StatusCode, header::AUTHORIZATION};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_json::json;
use ssh_key::PublicKey;
use uuid::Uuid;

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
    html_url: String,
    ssh_url: String,
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
                full_name: x.full_name,
                ssh_url: x.ssh_url,
                url: x.html_url,
            })
            .collect())
    }
    #[instrument(skip(self, config, access_token, public_key), err(Debug))]
    pub async fn add_ssh_key(
        &self,
        config: &ForgejoForgeConfig,
        repo: &UpstreamRepositoryInfo,
        access_token: &SecretString,
        public_key: &PublicKey,
    ) -> crate::Result<()> {
        let key = public_key
            .to_openssh()
            .map_err(|e| AppError::InternalError(e.into()))?;

        let res = HTTP
            .post(format!(
                "{}/api/v1/repos/{}/keys",
                &config.url, &repo.full_name,
            ))
            .header(
                AUTHORIZATION,
                format!("token {}", access_token.expose_secret()),
            )
            .json(&json!({
                "title": "Kanade CI",
                "read_only": false,
                "key": key,
            }))
            .send()
            .await?;

        let status = res.status();

        let text = res.text().await?;
        debug!(status = ?status, "response: {text}");

        if status != StatusCode::CREATED {
            return Err(AppError::InternalError(anyhow::anyhow!(
                "create webhook request failed with status {status}"
            )));
        }

        Ok(())
    }

    #[instrument(skip(self, config, access_token, secret), err(Debug))]
    pub async fn setup_webhook(
        &self,
        config: &ForgejoForgeConfig,
        access_token: &SecretString,
        repo: &UpstreamRepositoryInfo,
        secret: &str,
        repo_id: Uuid,
    ) -> crate::Result<()> {
        let res = HTTP.post(format!(
                "{}/api/v1/repos/{}/hooks",
                &config.url,
                &repo.full_name,
            ))
            .header(
                AUTHORIZATION,
                format!("token {}", access_token.expose_secret()),
            )
            .json(&json!({
                "active": true,
                "config": {
                    "url": format!("{}/_/webhook/forgejo?repo={}", &self.config.server.public_url, urlencoding::encode(&repo_id.to_string())),
                    "content_type": "json",
                    "secret": secret,
                },
                "events": [ "push", "create", "pull_request", "issues", "release" ],
                "type": "forgejo"
            }))
            .send()
            .await?;

        let status = res.status();

        let text = res.text().await?;
        debug!(status = ?status, "response: {text}");

        if status != StatusCode::CREATED {
            return Err(AppError::InternalError(anyhow::anyhow!(
                "create webhook request failed with status {status}"
            )));
        }

        Ok(())
    }

    pub async fn get_repo(
        &self,
        config: &ForgejoForgeConfig,
        access_token: &SecretString,
        repo_id: &str,
    ) -> crate::Result<Option<super::UpstreamRepositoryInfo>> {
        let res = match HTTP
            .get(format!(
                "{}/api/v1/repositories/{}",
                &config.url,
                urlencoding::encode(repo_id),
            ))
            .header(
                AUTHORIZATION,
                format!("token {}", access_token.expose_secret()),
            )
            .send()
            .await?
            .error_for_status()
        {
            Err(e) if e.status() == Some(StatusCode::NOT_FOUND) => {
                return Ok(None);
            }
            res => res,
        }?
        .json::<Repository>()
        .await?;

        Ok(Some(UpstreamRepositoryInfo {
            id: res.id.to_string(),
            full_name: res.full_name,
            url: res.html_url,
            ssh_url: res.ssh_url,
        }))
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
