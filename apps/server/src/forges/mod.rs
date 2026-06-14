use std::sync::Arc;

use chrono::Utc;
use oauth2::{RefreshToken, TokenResponse};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{PgPool, types::Json};
use ssh_key::PublicKey;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    crypto::CryptoEngine,
    data::forges::{ForgeConfig, ForgejoForgeConfig},
    error::AppError,
    forges::forgejo::ForgejoApi,
};

pub mod forgejo;

#[derive(Debug)]
pub struct UpstreamRepositoryInfo {
    pub id: String,
    pub full_name: String,
    pub url: String,
    pub _ssh_url: String,
}

pub struct AllForges {
    db: PgPool,
    crypto: Arc<CryptoEngine>,

    pub forgejo: ForgejoApi,
}

#[derive(Debug)]
pub enum ForgeAuthInfo {
    Forgejo {
        config: ForgejoForgeConfig,
        uid: String,
        access_token: SecretString,
    },
}

impl AllForges {
    pub fn new(
        config: Arc<AppConfig>,
        crypto: Arc<CryptoEngine>,
        db: PgPool,
    ) -> crate::Result<Self> {
        Ok(Self {
            db,
            crypto,
            forgejo: ForgejoApi::new(config)?,
        })
    }

    pub async fn get_forge_auth(
        &self,
        user_id: Uuid,
        forge_id: Uuid,
    ) -> crate::Result<Option<ForgeAuthInfo>> {
        let mut tx = self.db.begin().await?;

        let uf = match sqlx::query!(
            r#"
            SELECT uf.id, uf.access_token, uf.refresh_token, uf.access_token_expires_at, uf.forge_user_id, f.config as "forge_config: Json<ForgeConfig>"
            FROM user_forge uf
            LEFT JOIN forge f ON f.id = uf.forge_id
            WHERE
                user_id = $1 AND
                forge_id = $2
            FOR UPDATE OF uf
            "#,
            user_id,
            forge_id
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            Some(forge) => forge,
            None => return Ok(None),
        };

        let is_expired = Utc::now() > uf.access_token_expires_at;
        let at = self.crypto.decrypt(&uf.access_token)?;

        if !is_expired {
            return match uf.forge_config.0 {
                ForgeConfig::Forgejo(forgejo) => Ok(Some(ForgeAuthInfo::Forgejo {
                    config: forgejo,
                    uid: uf.forge_user_id,
                    access_token: at,
                })),
            };
        }

        let refresh_token = self.crypto.decrypt(&uf.refresh_token)?;

        let at = {
            let tokens = match &uf.forge_config.0 {
                ForgeConfig::Forgejo(forgejo) => {
                    self.forgejo
                        .refresh_token(
                            forgejo,
                            RefreshToken::new(refresh_token.expose_secret().to_string()),
                        )
                        .await?
                }
            };

            let at = SecretString::from(tokens.access_token().clone().into_secret());
            let rt = tokens
                .refresh_token()
                .ok_or(AppError::InvalidTokenResponse)?;
            let exp = Utc::now() + tokens.expires_in().ok_or(AppError::InvalidTokenResponse)?;

            sqlx::query!(
                r#"
                UPDATE user_forge
                SET
                    access_token = $1,
                    refresh_token = $2,
                    access_token_expires_at = $3
                WHERE
                    id = $4
                "#,
                self.crypto.encrypt(at.expose_secret())?,
                self.crypto.encrypt(rt.secret().as_str())?,
                exp,
                uf.id
            )
            .execute(&mut *tx)
            .await?;

            at
        };

        tx.commit().await?;

        match uf.forge_config.0 {
            ForgeConfig::Forgejo(forgejo) => Ok(Some(ForgeAuthInfo::Forgejo {
                config: forgejo,
                access_token: at,
                uid: uf.forge_user_id,
            })),
        }
    }

    pub async fn get_repo(
        &self,
        auth: &ForgeAuthInfo,
        repo_id: &str,
    ) -> crate::Result<Option<UpstreamRepositoryInfo>> {
        match auth {
            ForgeAuthInfo::Forgejo {
                config,
                access_token,
                ..
            } => self.forgejo.get_repo(&config, &access_token, repo_id).await,
        }
    }

    pub async fn get_repo_script(
        &self,
        auth: &ForgeAuthInfo,
        repo: &UpstreamRepositoryInfo,
        commit: &str,
    ) -> crate::Result<Option<String>> {
        match auth {
            ForgeAuthInfo::Forgejo {
                config,
                access_token,
                ..
            } => {
                self.forgejo
                    .get_repo_config(&config, &access_token, repo, commit)
                    .await
            }
        }
    }

    pub async fn add_ssh_key(
        &self,
        auth: &ForgeAuthInfo,
        repo: &UpstreamRepositoryInfo,
        public_key: &PublicKey,
    ) -> crate::Result<()> {
        match auth {
            ForgeAuthInfo::Forgejo {
                config,
                access_token,
                ..
            } => {
                self.forgejo
                    .add_ssh_key(&config, repo, &access_token, public_key)
                    .await
            }
        }
    }

    pub async fn setup_webhook(
        &self,
        auth: &ForgeAuthInfo,
        repo: &UpstreamRepositoryInfo,
        secret: &str,
        repo_id: Uuid,
    ) -> crate::Result<()> {
        match auth {
            ForgeAuthInfo::Forgejo {
                config,
                access_token,
                ..
            } => {
                self.forgejo
                    .setup_webhook(&config, &access_token, repo, secret, repo_id)
                    .await
            }
        }
    }

    pub async fn search_repositories(
        &self,
        user_id: Uuid,
        forge_id: Uuid,
        search: &str,
    ) -> crate::Result<Vec<UpstreamRepositoryInfo>> {
        let auth = self
            .get_forge_auth(user_id, forge_id)
            .await?
            .ok_or(AppError::ForgeNotLinked)?;

        match auth {
            ForgeAuthInfo::Forgejo {
                config,
                uid,
                access_token,
            } => {
                self.forgejo
                    .search_repositories(&config, &uid, &access_token, search)
                    .await
            }
        }
    }
}
