use crate::auth::Auth;
use axum::extract::Query;
use axum::{response::IntoResponse, response::Redirect, BoxError, Extension};
use oauth2::TokenResponse;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use serde::Serialize;
use tower_sessions::Session;

const USER_KEY: &str = "user";
const CRSF_TOKEN: &str = "csrf";

// For testing to avoid actually hit the GitHub API constantly while tinkering
const PRETEND_TO_LOGIN: bool = false;

#[derive(Default, Deserialize, Serialize)]
struct User {
    id: i64,
    email: String,
    token: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("token", &"[redacted]")
            .finish()
    }
}

pub async fn github_auth_handler(
    Extension(auth): Extension<Auth>,
    session: Session,
) -> impl IntoResponse {
    tracing::info!("github_auth_handler: session: {:?}", session.id());

    // Pretend we logged in
    if PRETEND_TO_LOGIN {
        let user = User {
            id: 1, // This should be a real ID
            email: "test".to_string(),
            token: "token".to_string(),
        };
        session.insert(USER_KEY, user).await.unwrap();
        tracing::info!("Bypassing login, prtending it worked");
        log_user_session(&session).await;
    }

    // Check if the user is already logged in
    if logged_in(&session).await {
        tracing::info!("Already logged in, skipping GitHub auth");
        return Redirect::temporary("/");
    }

    // Generate the authorization URL
    let (authorize_url, csrf_state) = auth.generate_auth_url();

    session.insert(CRSF_TOKEN, csrf_state).await.unwrap();

    tracing::info!("Going to redirect to GitHub!");
    Redirect::temporary(authorize_url.as_str())
}

#[derive(Debug, Deserialize)]
pub struct CallbackParameters {
    code: String,
    state: String,
}

pub async fn github_login_callback(
    Extension(auth): Extension<Auth>,
    Query(response): Query<CallbackParameters>,
    session: Session,
) -> impl IntoResponse {
    tracing::info!("github_login_callback: session: {:?}", session.id());

    // Check the CSRF state, this is to prevent CSRF attacks.
    // The state must exist, AND be the same as the one we stored in the session.
    let state = response.state;
    let csrf_state: Option<String> = session.get(CRSF_TOKEN).await.unwrap();
    if csrf_state.is_none() {
        tracing::error!("No CSRF token in session!");
        return Redirect::temporary("/");
    }
    if state != csrf_state.unwrap() {
        tracing::error!("CSRF token mismatch!");
        return Redirect::temporary("/");
    }

    // Exchange the code with a token and store it in the session for future use
    let code = response.code;
    let token_response = auth.exchange_code(code).await;
    let token = token_response.access_token();

    // Use the token to get the user email
    let user_email = get_user_email(token.secret()).await.unwrap();
    tracing::info!("Got user email! {:?}", user_email.to_string());

    // Store the user in the session
    let user = User {
        id: 1, // This should be a real ID
        email: user_email,
        token: token.secret().to_string(), // store the token in a cookie?
    };
    session.insert(USER_KEY, user).await.unwrap();

    log_user_session(&session).await;

    Redirect::temporary("/")
}

pub async fn log_user_session(session: &Session) {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    tracing::info!("User: {:?}", user);
}

// This is hack, need cookie management and all that
pub async fn logged_in(session: &Session) -> bool {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    user.is_some()
}

pub async fn get_user_email_from_session(session: &Session) -> String {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    if user.is_some() {
        let user = user.unwrap();
        return user.email;
    }
    return "no email".to_string();
}

#[derive(Debug, Deserialize)]
struct Email {
    email: String,
    primary: bool,
    verified: bool,
}

// This is a first draft, need to handle errors properly, maybe put this in another mod
pub async fn get_user_email(token: &str) -> Result<String, BoxError> {
    let user_emails_url = "https://api.github.com/user/emails";

    let email_response: reqwest::Response = ReqwestClient::new()
        .get(user_emails_url)
        .header("User-Agent", "Eventageous")
        .header("Accept", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    //tracing::info!("Got email_response from GitHub! {:?}", email_response);
    match email_response.status() {
        reqwest::StatusCode::OK => {
            tracing::info!("Got emails from GitHub, will try to parse");
            // Handle the successful response here
            let emails = email_response.text().await.unwrap();
            tracing::info!("JSON {:?}", emails);
            let emails: Vec<Email> = serde_json::from_str(&emails).unwrap();

            // check for primary and verified emails  and return the first one
            for email in emails {
                if email.primary && email.verified {
                    return Ok(email.email);
                }
            }
        }
        reqwest::StatusCode::FORBIDDEN => {
            tracing::error!("Received a 403 Forbidden response");
            if let Some(rate_limit_remaining) =
                email_response.headers().get("X-RateLimit-Remaining")
            {
                if rate_limit_remaining == "0" {
                    let rate_limit_reset =
                        email_response.headers().get("X-RateLimit-Reset").unwrap();
                    let reset_time = std::time::UNIX_EPOCH
                        + std::time::Duration::from_secs(
                            rate_limit_reset.to_str().unwrap().parse::<u64>().unwrap(),
                        );
                    tracing::error!("Rate limit exceeded, will reset at {:?}", reset_time);
                    // Handle the rate limit exceeded case here
                }
            }
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            tracing::error!("Received a 401 Unauthorized response");
            // Handle the 401 Unauthorized response here
        }
        _ => {
            tracing::error!(
                "Received an unexpected HTTP response: {}",
                email_response.status()
            );
            // Handle other unexpected responses here
        }
    }

    /* Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to get user emails",
    )))*/

    return Ok("no email".to_string());
}
