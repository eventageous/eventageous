use axum::extract::Query;
use axum::{
    error_handling::HandleErrorLayer, extract::State, response::IntoResponse, response::Redirect,
    routing::get, BoxError, Extension, Json, Router,
};

use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use serde::Serialize;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use time::Duration;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use unterstutzen::Calendar;
use unterstutzen::Configuration;
use unterstutzen::Events;
use unterstutzen::OAuthConfig;

const TOKEN_KEY: &str = "token";
const USER_KEY: &str = "user";
const CRSF_TOKEN: &str = "csrf";

// For testing, should be defined on a cookie or something
const SESSION_LENGTH_SECONDS: i64 = 60 * 2;

// For testing to avoid actually hit the GitHub API constantly while tinkering
const PRETEND_TO_LOGIN: bool = false;

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
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
    let client = create_basic_client_from_config(&oauth_config);

    let session_store = MemoryStore::default();

    // Create a route for the GitHub auth handler
    let auth_router = Router::new()
        .route("/login", get(github_auth_handler))
        .route("/callback", get(github_login_callback))
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

fn create_basic_client_from_config(oauth_config: &OAuthConfig) -> BasicClient {
    let client_id = ClientId::new(oauth_config.client_id.to_string());
    let client_secret = ClientSecret::new(oauth_config.client_secret.to_string());
    let auth_url = AuthUrl::new(oauth_config.auth_url.to_string()).unwrap();
    let token_url = TokenUrl::new(oauth_config.token_url.to_string()).unwrap();

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
}

#[derive(Debug, Serialize)]
struct Response {
    data: Events,
    authed: bool, // don't do this for realz, just for testing
}

async fn handler(State(config): State<Arc<Configuration>>, session: Session) -> Json<Response> {
    tracing::info!("handler: session: {:?}", session.id());

    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();
    tracing::info!("Got data from Calenar API!");
    //tracing::info!("{}", serde_json::to_string_pretty(&response).unwrap());

    let logged_in = logged_in(&session).await;
    tracing::info!("Is user logged in: {}", logged_in);
    let response = Response {
        data: events,
        authed: logged_in,
    };

    Json(response)
}
// This is hack, need cookie management and all that
async fn logged_in(session: &Session) -> bool {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    user.is_some()
}

#[derive(Default, Deserialize, Serialize)]
struct User {
    name: String,
    token: String,
}

async fn github_auth_handler(
    Extension(client): Extension<BasicClient>,
    session: Session,
) -> impl IntoResponse {
    tracing::info!("github_auth_handler: session: {:?}", session.id());

    // Pretend we logged in
    if PRETEND_TO_LOGIN {
        let user = User {
            name: "test".to_string(),
            token: "token".to_string(),
        };
        session.insert(USER_KEY, user).await.unwrap();
        tracing::info!("Bypassing login, prtending it worked");
    }

    // Check if the user is already logged in
    if logged_in(&session).await {
        tracing::info!("Already logged in, skipping GitHub auth");
        return Redirect::temporary("/");
    }

    // Create an OAuth2 client
    tracing::info!("Going to redirect to GitHub!");

    // Generate the authorization URL
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // Add the email scope
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    session.insert(CRSF_TOKEN, csrf_state).await.unwrap();

    Redirect::temporary(authorize_url.as_str())
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn github_login_callback(
    Extension(client): Extension<BasicClient>,
    Query(params): Query<CallbackQuery>,
    session: Session,
) -> impl IntoResponse {
    tracing::info!("github_login_callback: session: {:?}", session.id());
    tracing::info!("Callback from GitHub! {} / {}", params.code, params.state);

    let code = params.code;
    let state = params.state;

    let csrf_state: String = session.get(CRSF_TOKEN).await.unwrap().unwrap();
    if state != csrf_state {
        tracing::error!("CSRF token mismatch!");
        return Redirect::temporary("/auth/login");
    }

    // Exchange the code with a token and store it in the session
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .unwrap();

    let token = token.access_token().secret();
    let user = User {
        name: "test".to_string(),
        token: token.to_string(),
    };

    tracing::info!("Got token from GitHub! {:?}", "secret!");

    session.insert(USER_KEY, user).await.unwrap();

    Redirect::temporary("/")
}
