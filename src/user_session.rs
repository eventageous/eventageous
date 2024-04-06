use crate::auth::{Auth, CallbackState};
use axum::extract::Query;
use axum::{response::IntoResponse, response::Redirect, Extension};
use serde::Deserialize;
use serde::Serialize;
use tower_sessions::Session;

const USER_KEY: &str = "user";
const AUTH_STATE: &str = "auth_state";

// For testing to avoid actually hitting the GitHub API constantly while tinkering
const PRETEND_TO_LOGIN: bool = false;

#[derive(Default, Deserialize, Serialize)]
struct User {
    id: i64,
    email: String,
}

// Don't need this really unless we store the token, leaving for now
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            //.field("token", &"[redacted]")
            .finish()
    }
}

pub async fn login_handler(
    Extension(auth): Extension<Auth>,
    session: Session,
) -> impl IntoResponse {
    tracing::info!("login_handler: session: {:?}", session.id());

    // Pretend we logged in
    if PRETEND_TO_LOGIN {
        pretend_login(session).await;
        return Redirect::temporary("/");
    }

    // Check if the user is already logged in
    if logged_in(&session).await {
        tracing::info!("Already logged in, skipping GitHub auth");
        return Redirect::temporary("/");
    }

    // Generate the authorization URL
    let (authorize_url, auth_state) = auth.generate_auth_url();
    session.insert(AUTH_STATE, auth_state).await.unwrap();

    tracing::info!("Redirecting to GitHub auth!");
    Redirect::temporary(authorize_url.as_str())
}

pub async fn pretend_login(session: Session) {
    let user = User {
        id: 1, // This should be a real ID
        email: "test_at_boop".to_string(),
    };
    session.insert(USER_KEY, user).await.unwrap();
    tracing::info!("Bypassing login, prtending it worked");
    log_user_session(&session).await;
}

pub async fn github_login_callback(
    Extension(auth): Extension<Auth>,
    Query(callback_state): Query<CallbackState>,
    session: Session,
) -> impl IntoResponse {
    tracing::info!("github_login_callback: session: {:?}", session.id());

    let auth_state = session.get(AUTH_STATE).await.unwrap();
    let user_email = auth.authenticate(auth_state, callback_state).await;

    // If there is email returned, authentication failed, redirect
    if user_email.is_none() {
        return Redirect::temporary("/");
    }

    // Store the user in the session
    let user = User {
        id: 1, // This should be a real ID
        email: user_email.unwrap(),
    };

    session.insert(USER_KEY, user).await.unwrap();
    log_user_session(&session).await;

    Redirect::temporary("/")
}

pub async fn log_user_session(session: &Session) {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    tracing::info!("User: {:?}", user);
}

// TODO: need cookie management and all that
pub async fn logged_in(session: &Session) -> bool {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    user.is_some()
}

// For testing
pub async fn get_user_email_from_session(session: &Session) -> String {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    if user.is_some() {
        let user = user.unwrap();
        return user.email;
    }
    return "no email, user may not be logged in".to_string();
}
