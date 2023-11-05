use axum::{routing::get, Router};

use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use std::sync::Arc;
use unterstutzen::Calendar;
use unterstutzen::Configuration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).pretty().init();

    let app = Router::new().route("/", get(handler)).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    tracing::info!("Listening on localhost:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    let config = Arc::new(Configuration::load().unwrap());
    let calendar = Calendar::from(&config);
    let events = calendar.events().await.unwrap();
    tracing::info!("{events:#?}");

    let response = "Check your console output!";
    tracing::info!("handler: {}", response);

    response
}
