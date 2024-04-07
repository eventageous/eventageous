use crate::auth;
use crate::oauth_config::OAuthConfig;
use axum::BoxError;
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use oauth2::{
    basic::BasicClient, basic::BasicTokenType, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, EmptyExtraTokenFields, Scope, StandardTokenResponse, TokenUrl,
};
use reqwest::Client as ReqwestClient;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Auth {
    client: BasicClient,
}

#[derive(Serialize, Deserialize)]
pub struct AuthState {
    pub csrf_state: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackState {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct Email {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub email: String,
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
    pub fn generate_auth_url(&self) -> (Url, AuthState) {
        let (url, crsf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user:email".to_string()))
            .url();
        (
            url,
            AuthState {
                csrf_state: crsf_token.secret().clone(),
            },
        )
    }

    pub async fn authenticate(
        &self,
        auth_state: Option<AuthState>,
        callback_state: CallbackState,
    ) -> Option<AuthenticatedUser> {
        // Validate state to prevent
        let state = callback_state.state;
        let state_is_valid = self.validate_state(state.clone(), auth_state);
        if !state_is_valid {
            // TODO: probably real error handling, for now, return this and we'll always redirect
            return None;
        }

        // Exchange the code with a token and store it in the session for future use
        let code = callback_state.code;
        let token_response = self.exchange_code(code).await;
        let token = token_response.access_token();

        // NOTE: If we want to take more actions on behalf of the user in GitHub, then we could store this
        // token in the session and use it to make requests to the GitHub API.
        // For now, we just want the email to identify the user, and we're not storing the token
        // since we're not doing anything else with it and it is safer to not until we need it.

        // Use the token to get the user email
        let user_email = self
            .get_authenticated_user_email(token.secret().as_str())
            .await
            .unwrap();
        tracing::info!("Got user email! {:?}", user_email.to_string());

        let authenticated_user = AuthenticatedUser { email: user_email };

        return Option::Some(authenticated_user);
    }

    // Check the CSRF state, this is to prevent CSRF attacks.
    // The state must exist, AND be the same as the one we stored in the session.
    fn validate_state(&self, response_state: String, expected_state: Option<AuthState>) -> bool {
        if expected_state.is_none() {
            tracing::error!("No CSRF token in session!");
        } else if response_state != expected_state.unwrap().csrf_state {
            tracing::error!("CSRF token mismatch!");
        } else {
            return true;
        }
        return false;
    }

    async fn exchange_code(
        &self,
        code: String,
    ) -> StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> {
        self.client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .unwrap()
    }

    async fn get_authenticated_user_email(&self, token: &str) -> anyhow::Result<String> {
        let user_emails_url = "https://api.github.com/user/emails";

        let email_response = self.send_request(user_emails_url, token).await?;

        tracing::info!("Got emails from GitHub, will try to parse");

        let emails = email_response.text().await.unwrap();
        tracing::info!("JSON {:?}", emails);
        let emails: Vec<Email> = serde_json::from_str(&emails).unwrap();
        // check for primary and verified emails  and return the first match
        for email in emails {
            if email.primary && email.verified {
                return Ok(email.email);
            }
        }

        // TODO: If no email, return an error
        return Ok("no email!".to_string());
    }

    async fn send_request(&self, url: &str, token: &str) -> anyhow::Result<reqwest::Response> {
        let response: reqwest::Response = ReqwestClient::new()
            .get(url)
            .header("User-Agent", "Eventageous")
            .header("Accept", "application/json")
            .bearer_auth(token)
            .send()
            .await
            .unwrap();

        match response.status() {
            reqwest::StatusCode::OK => {
                tracing::info!("Got response from GitHub");
                return Ok(response);
            }
            reqwest::StatusCode::FORBIDDEN => {
                tracing::error!("Received a 403 Forbidden response");
                if let Some(rate_limit_remaining) = response.headers().get("X-RateLimit-Remaining")
                {
                    if rate_limit_remaining == "0" {
                        let rate_limit_reset = response.headers().get("X-RateLimit-Reset").unwrap();
                        let reset_time = std::time::UNIX_EPOCH
                            + std::time::Duration::from_secs(
                                rate_limit_reset.to_str().unwrap().parse::<u64>().unwrap(),
                            );
                        tracing::error!("Rate limit exceeded, will reset at {:?}", reset_time);
                        // Handle the rate limit exceeded case here if needed
                    }
                }
                return Err(anyhow::anyhow!("Received a 403 Forbidden response"));
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                tracing::error!("Received a 401 Unauthorized response");
                return Err(anyhow::anyhow!("Received a 401 Forbidden response"));
            }
            _ => {
                tracing::error!(
                    "Received an unexpected HTTP response: {}",
                    response.status()
                );
                return Err(anyhow::anyhow!("Received an unexpected HTTP response"));
            }
        }
    }
}
