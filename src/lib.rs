use axum::{extract::State, routing::get, Extension, Json, Router};

use calendar::Events;
use config::Configuration;
use oauth_config::OAuthConfig;
use serde::Serialize;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use time::Duration;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

use crate::calendar::Calendar;

mod calendar;
mod config;
mod oauth_config;
mod user_auth;
// For testing, should be defined on a cookie or something
const SESSION_LENGTH_SECONDS: i64 = 60 * 2;

pub async fn eventageous(secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    // Configure the backend
    let google_api_key = secret_store.get("GOOGLE_API_KEY").unwrap();
    let google_calendar_id = secret_store.get("GOOGLE_CALENDAR_ID").unwrap();
    let config = Arc::new(Configuration::new(google_api_key, google_calendar_id));

    // Configure OAuth
    let oauth2_client_id = secret_store.get("GITHUB_CLIENT_ID").unwrap();
    let oauth_client_secret = secret_store.get("GITHUB_CLIENT_SECRET").unwrap();
    let oauth2_callback_url = secret_store.get("GITHUB_CALLBACK_URL").unwrap();

    let oauth_config = Arc::new(OAuthConfig::new(
        oauth2_client_id.to_string(),
        oauth_client_secret.to_string(),
        "https://github.com/login/oauth/authorize".to_string(),
        "https://github.com/login/oauth/access_token".to_string(),
        oauth2_callback_url.to_string(),
    ));
    let client = user_auth::create_basic_client_from_config(&oauth_config);

    let session_store = MemoryStore::default();

    // Create a route for the GitHub auth handler
    let auth_router = Router::new()
        .route("/login", get(user_auth::github_auth_handler))
        .route("/callback", get(user_auth::github_login_callback))
        .layer(Extension(client));

    // Configure the routes
    let router = Router::new()
        .nest_service("/", ServeDir::new("dist"))
        .route("/api/events", get(handler))
        .nest("/auth", auth_router)
        .with_state(config)
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(
                    SESSION_LENGTH_SECONDS,
                ))),
        );

    Ok(router.into())
}

#[derive(Debug, Serialize)]
struct Response {
    data: Events,
    authed: bool, // don't do this for realz, just for testing
    email: String,
}

async fn handler(State(config): State<Arc<Configuration>>, session: Session) -> Json<Response> {
    tracing::info!("handler: session: {:?}", session.id());

    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();
    tracing::info!("Got data from Calenar API!");

    // Shoving this data in this response for now, should handle properly
    let logged_in = user_auth::logged_in(&session).await;
    let email = user_auth::get_user_email_from_session(&session).await;
    tracing::info!("Logged in {} /email {}", logged_in, email);
    let response = Response {
        data: events,
        authed: logged_in,
        email: email,
    };

    Json(response)
}
