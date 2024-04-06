use crate::auth::Auth;
use axum::extract::Query;
use axum::{response::IntoResponse, response::Redirect, Extension};
use oauth2::TokenResponse;
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

    // Validate state to prevent CSRF attacks
    let state = response.state;
    let state_is_valid = auth.validate_state(state.clone(), session.get(CRSF_TOKEN).await.unwrap());
    if !state_is_valid {
        return Redirect::temporary("/");
    }

    // Exchange the code with a token and store it in the session for future use
    let code = response.code;
    let token_response = auth.exchange_code(code).await;
    let token = token_response.access_token();

    // Use the token to get the user email
    let user_email = auth
        .get_authenticated_user_email(token.secret())
        .await
        .unwrap();
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
