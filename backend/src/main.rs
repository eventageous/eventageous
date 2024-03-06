use axum::{routing::get, Json, Router};

use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use serde::Serialize;
use std::sync::Arc;
use unterstutzen::Calendar;
use unterstutzen::Configuration;
use unterstutzen::Events;

#[derive(Debug, Serialize)]
struct Response {
    data: Events,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).pretty().init();

    let app = Router::new().route("/", get(handler)).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    tracing::info!("Listening on localhost:3300");

    axum::Server::bind(&"0.0.0.0:3300".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Json<Response> {
    let config = Arc::new(Configuration::load().unwrap());
    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();

    // pretend it's always sucessfull
    let response = Response { data: events };

    //tracing::info!("{}", serde_json::to_string_pretty(&response).unwrap());

    Json(response)
}
