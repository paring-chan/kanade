use oauth2::{basic::*, *};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ForgeConfig {
    Forgejo(ForgejoForgeConfig),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgejoForgeConfig {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
}

impl ForgejoForgeConfig {
    pub fn oauth2_client(
        &self,
        config: &AppConfig,
    ) -> crate::Result<
        Client<
            StandardErrorResponse<BasicErrorResponseType>,
            StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
            StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
            StandardRevocableToken,
            StandardErrorResponse<RevocationErrorResponseType>,
            EndpointSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointSet,
        >,
    > {
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_redirect_uri(RedirectUrl::new(self.url.clone())?)
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(format!(
                "{}/login/oauth/authorize",
                &self.url
            ))?)
            .set_redirect_uri(RedirectUrl::new(format!(
                "{}/_/auth/callback",
                &config.server.public_url
            ))?)
            .set_token_uri(TokenUrl::new(format!(
                "{}/login/oauth/access_token",
                &self.url
            ))?);

        Ok(client)
    }
}
