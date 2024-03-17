use axum::extract::Query;
use axum::{
    extract::State, response::IntoResponse, response::Redirect, routing::get, Extension, Json,
    Router,
};
//use axum_extra::extract::{cookie, cookie::Key};
use async_session::{MemoryStore, Session};
use oauth2::basic::BasicClient;
use oauth2::http::Request;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, Scope, TokenUrl,
};
use serde::Deserialize;
use serde::Serialize;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use tower_http::services::ServeDir;
use unterstutzen::Calendar;
use unterstutzen::Configuration;
use unterstutzen::Events;
use unterstutzen::OAuthConfig;

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
        .with_state(config);

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
}

async fn handler(State(config): State<Arc<Configuration>>) -> Json<Response> {
    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();

    // pretend it's always sucessfull
    let response = Response { data: events };

    tracing::info!("Got data from Google API!");
    //tracing::info!("{}", serde_json::to_string_pretty(&response).unwrap());

    Json(response)
}

async fn github_auth_handler(Extension(client): Extension<BasicClient>) -> impl IntoResponse {
    // Create an OAuth2 client
    tracing::info!("Going to redirect to GitHub!");

    // Generate the authorization URL
    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // Add the email scope
        .add_scope(Scope::new("user:email".to_string()))
        .url();

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
) -> impl IntoResponse {
    tracing::info!("Callback from GitHub! {} / {}", params.code, params.state);

    let code = params.code;
    let state = params.state;

    // Exchange the code with a token and store it in the session
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .unwrap();

    tracing::info!("Got token from GitHub! {:?}", token);

    // Store the token in the session, and make the frontend know what's up
    //session.insert("github_token", token.access_token().secret().to_string());

    // TODO: Store the token in the session
    Redirect::temporary("/")
}
