use axum::{routing::get, Json, Router};
use tower_http::services::ServeDir;
use serde::Serialize;
use std::sync::Arc;
use unterstutzen::Calendar;
use unterstutzen::Configuration;
use unterstutzen::Events;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .nest_service("/", ServeDir::new("assets"))
        .route("/api/events", get(handler));

    Ok(router.into())
}

#[derive(Debug, Serialize)]
struct Response {
    data: Events,
}

async fn handler() -> Json<Response> {
    let config = Arc::new(Configuration::load().unwrap());
    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();

    // pretend it's always sucessfull
    let response = Response { data: events };

    tracing::info!("Got data from Google AP!");
    //tracing::info!("{}", serde_json::to_string_pretty(&response).unwrap());

    Json(response)
}
