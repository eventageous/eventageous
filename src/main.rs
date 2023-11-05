use axum::{routing::get, Router};

use tower_http::trace::{self, TraceLayer};
use tracing::Level;

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
    let response = "Best response ever!";
    tracing::info!("handler: {}", response);
    response
}
