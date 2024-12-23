//! A sync server for beeps.

use axum::{http::header::AUTHORIZATION, response::IntoResponse, routing::get, Router};
use clap::Parser;
use std::{iter::once, time::Duration};
use tokio::net::TcpListener;
use tower_http::{compression, decompression, limit, sensitive_headers, timeout, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Config {
    #[clap(long, env, default_value = "127.0.0.1:3000")]
    address: String,

    /// Request body size limit, in bytes
    #[clap(long, env, default_value = "5242880")]
    body_limit: usize,

    /// Request timeout, in seconds
    #[clap(long, env, default_value = "5", value_parser = duration_parser)]
    request_timeout: Duration,

    #[clap(long, env)]
    jwt_secret: String,

    #[clap(long, env)]
    login_password: String,
}

fn duration_parser(s: &str) -> Result<Duration, std::num::ParseIntError> {
    s.parse().map(Duration::from_secs)
}

#[tokio::main]
async fn main() {
    let options = Config::parse();

    // TODO: opentelemetry
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .layer(trace::TraceLayer::new_for_http())
        .layer(compression::CompressionLayer::new())
        .layer(decompression::DecompressionLayer::new())
        .layer(limit::RequestBodyLimitLayer::new(options.body_limit))
        .layer(sensitive_headers::SetSensitiveHeadersLayer::new(once(
            AUTHORIZATION,
        )))
        .layer(timeout::TimeoutLayer::new(options.request_timeout))
        // ROUTES
        .route("/", get(handler));
    // STATE
    // .with_state(state);

    let listener = TcpListener::bind(options.address).await.unwrap();
    tracing::info!(address = ?listener.local_addr(), "listening");

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> impl IntoResponse {
    "Hello, World!"
}
