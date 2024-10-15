use std::{iter::once, time::Duration};

use axum::{http::header::AUTHORIZATION, routing::get, Router};
use clap::Parser;
use tower_http::{compression, limit, sensitive_headers, timeout, trace};
use tracing::level_filters::LevelFilter;

/// Keep track of what you're doing throughout the day by being annoyed by a robot.
#[derive(Parser, Debug)]
#[clap(version)]
struct Options {
    #[clap(long = "log-level", default_value = "info", env = "LOG_LEVEL")]
    log_level: LevelFilter,

    #[clap(long = "address", default_value = "0.0.0.0:3000", env = "ADDRESS")]
    address: String,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    // TODO: opentelemetry
    tracing_subscriber::fmt()
        .with_max_level(options.log_level)
        .init();

    let app = Router::new()
        .layer(trace::TraceLayer::new_for_http())
        .layer(compression::CompressionLayer::new())
        .layer(limit::RequestBodyLimitLayer::new(1024 * 1024 * 5))
        .layer(sensitive_headers::SetSensitiveHeadersLayer::new(once(
            AUTHORIZATION,
        )))
        .layer(timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .route("/", get(|| async { "Hello, World!" }));

    tracing::info!(address = &options.address, "listening");
    let listener = tokio::net::TcpListener::bind(options.address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
