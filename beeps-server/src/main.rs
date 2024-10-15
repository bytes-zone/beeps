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

    /// Request body size limit, in bytes
    #[clap(long = "body-limit", default_value = "5242880", env = "BODY_LIMIT")]
    body_limit: usize,

    /// Request timeout, in seconds
    #[clap(long = "request-timeout", value_parser = duration_parser, default_value = "5", env = "REQUEST_TIMEOUT")]
    request_timeout: Duration,
}

fn duration_parser(s: &str) -> Result<Duration, std::num::ParseIntError> {
    s.parse().map(Duration::from_secs)
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
        .layer(limit::RequestBodyLimitLayer::new(options.body_limit))
        .layer(sensitive_headers::SetSensitiveHeadersLayer::new(once(
            AUTHORIZATION,
        )))
        .layer(timeout::TimeoutLayer::new(options.request_timeout))
        .route("/", get(|| async { "Hello, World!" }));

    tracing::info!(address = &options.address, "listening");
    let listener = tokio::net::TcpListener::bind(options.address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
