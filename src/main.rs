use axum::{extract::State, routing::get, Json, Router};
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
        .nest_service("/", ServeDir::new("assets"))
        .route("/api/events", get(handler))
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
