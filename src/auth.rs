use crate::oauth_config::OAuthConfig;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Auth {
    pub client: BasicClient,
    pub config: Arc<OAuthConfig>,
}

impl From<Arc<OAuthConfig>> for Auth {
    fn from(config: Arc<OAuthConfig>) -> Self {
        let client = create_basic_client_from_config(&config);
        Self { client, config }
    }
}

pub fn create_basic_client_from_config(oauth_config: &OAuthConfig) -> BasicClient {
    let client_id = ClientId::new(oauth_config.client_id.to_string());
    let client_secret = ClientSecret::new(oauth_config.client_secret.to_string());
    let auth_url = AuthUrl::new(oauth_config.auth_url.to_string()).unwrap();
    let token_url = TokenUrl::new(oauth_config.token_url.to_string()).unwrap();

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
}
