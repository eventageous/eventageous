use crate::oauth_config::OAuthConfig;
use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, basic::BasicTokenType, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, EmptyExtraTokenFields, Scope, StandardTokenResponse, TokenUrl,
};

use reqwest::Url;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Auth {
    client: BasicClient,
}

impl From<Arc<OAuthConfig>> for Auth {
    fn from(config: Arc<OAuthConfig>) -> Self {
        let client_id = ClientId::new(config.client_id.to_string());
        let client_secret = ClientSecret::new(config.client_secret.to_string());
        let auth_url = AuthUrl::new(config.auth_url.to_string()).unwrap();
        let token_url = TokenUrl::new(config.token_url.to_string()).unwrap();

        let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));
        Self { client }
    }
}

impl Auth {
    pub fn generate_auth_url(&self) -> (Url, CsrfToken) {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user:email".to_string()))
            .url()
    }

    pub async fn exchange_code(
        &self,
        code: String,
    ) -> StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> {
        self.client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .unwrap()
    }
}
