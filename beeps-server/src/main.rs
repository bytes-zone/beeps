//! A sync server for beeps.

use axum::{http::header::AUTHORIZATION, response::IntoResponse, routing::get, Router};
use clap::Parser;
use std::{iter::once, num::ParseIntError, time::Duration};
use tokio::net::TcpListener;
use tower_http::{compression, decompression, limit, sensitive_headers, timeout, trace};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Configuration for the server
#[derive(Debug, Parser)]
#[clap(version)]
struct Config {
    /// On what interface and port to listen
    #[clap(long, env, default_value = "127.0.0.1:3000")]
    address: String,

    /// Request body size limit, in bytes
    #[clap(long, env, default_value = "5242880")]
    body_limit: usize,

    /// Request timeout, in seconds
    #[clap(long, env, default_value = "5", value_parser = duration_parser)]
    request_timeout: Duration,

    /// Secret to use to sign JWTs
    #[clap(long, env)]
    jwt_secret: String,

    /// Password to use for logging in
    #[clap(long, env)]
    login_password: String,
}

/// Parse a duration from a string
fn duration_parser(s: &str) -> Result<Duration, ParseIntError> {
    s.parse().map(Duration::from_secs)
}

#[tokio::main]
async fn main() {
    let options = Config::parse();

    // TODO: opentelemetry
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("BEEPS_LOG")
                .from_env_lossy(),
        )
        .with(fmt::layer())
        .init();

    let app = Router::new()
        // ROUTES
        .route("/", get(handler))
        // STATE
        // .with_state(state);
        // MIDDLEWARE
        .layer(trace::TraceLayer::new_for_http())
        .layer(compression::CompressionLayer::new())
        .layer(decompression::DecompressionLayer::new())
        .layer(limit::RequestBodyLimitLayer::new(options.body_limit))
        .layer(sensitive_headers::SetSensitiveHeadersLayer::new(once(
            AUTHORIZATION,
        )))
        .layer(timeout::TimeoutLayer::new(options.request_timeout));

    let listener = TcpListener::bind(options.address).await.unwrap();
    tracing::info!(address = ?listener.local_addr(), "listening");

    axum::serve(listener, app).await.unwrap();
}

/// Just standing in for a real handler during setup
async fn handler() -> impl IntoResponse {
    "Hello, World!"
}
