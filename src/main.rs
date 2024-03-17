use axum::extract::Query;
use axum::{
    extract::State, response::IntoResponse, response::Redirect, routing::get, BoxError, Extension,
    Json, Router,
};
use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    Scope, TokenResponse, TokenUrl,
};
use reqwest::Client as ReqwestClient;
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
    email: String,
}

async fn handler(State(config): State<Arc<Configuration>>, session: Session) -> Json<Response> {
    tracing::info!("handler: session: {:?}", session.id());

    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();
    tracing::info!("Got data from Calenar API!");

    // Shoving this data in this response for now, should handle properly
    let logged_in = logged_in(&session).await;
    let email = get_user_email_from_session(&session).await;
    tracing::info!("Logged in {} /email {}", logged_in, email);
    let response = Response {
        data: events,
        authed: logged_in,
        email: email,
    };

    Json(response)
}
// This is hack, need cookie management and all that
async fn logged_in(session: &Session) -> bool {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    user.is_some()
}

async fn get_user_email_from_session(session: &Session) -> String {
    let user: Option<User> = session.get(USER_KEY).await.unwrap();
    if user.is_some() {
        let user = user.unwrap();
        return user.email;
    }
    return "no email".to_string();
}

#[derive(Default, Deserialize, Serialize)]
struct User {
    email: String,
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
            email: "test".to_string(),
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

    let code = params.code;
    let state = params.state;

    let csrf_state: Option<String> = session.get(CRSF_TOKEN).await.unwrap();
    if csrf_state.is_none() {
        tracing::error!("No CSRF token in session!");
        return Redirect::temporary("/");
    }
    if state != csrf_state.unwrap() {
        tracing::error!("CSRF token mismatch!");
        return Redirect::temporary("/");
    }

    // Exchange the code with a token and store it in the session
    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .unwrap();

    let token = token_response.access_token();
    let user_email = get_user_email(token.secret()).await.unwrap();

    tracing::info!("Got user email! {:?}", user_email.to_string());

    let user = User {
        email: user_email,
        token: "pretend-token".to_string(), // store the token in a cookie, not here in plain text
    };
    session.insert(USER_KEY, user).await.unwrap();

    Redirect::temporary("/")
}

#[derive(Debug, Deserialize)]
struct Email {
    email: String,
    primary: bool,
    verified: bool,
}

// This is a first draft, need to handle errors properly, maybe put this in another mod
async fn get_user_email(token: &str) -> Result<String, BoxError> {
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
