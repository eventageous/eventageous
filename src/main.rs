use axum::{extract::State, http::Uri, response::Redirect, routing::get, Json, Router};
use serde::Serialize;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use tower_http::services::ServeDir;
use unterstutzen::Calendar;
use unterstutzen::Configuration;
use unterstutzen::Events;

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    // Configure the backend
    let google_api_key = secret_store.get("GOOGLE_API_KEY").unwrap();
    let google_calendar_id = secret_store.get("GOOGLE_CALENDAR_ID").unwrap();
    let config = Arc::new(Configuration::new(google_api_key, google_calendar_id));

    let router = Router::new()
        .nest_service("/", ServeDir::new("dist"))
        .route("/api/events", get(handler))
        .route("/auth/login", get(github_auth_handler))
        .route("/auth/callback", get(github_login_callback))
        .with_state(config);

    Ok(router.into())
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

async fn github_auth_handler(State(config): State<Arc<Configuration>>) -> Redirect {
    let client_id = "not_a_client_id";
    tracing::info!("Pretending to redirect to GitHub with {}!", client_id);

    let uri = "/auth/callback";
    Redirect::temporary(uri)
}

async fn github_login_callback(State(config): State<Arc<Configuration>>) {
    tracing::info!("Pretending we had a successful login!");
}
