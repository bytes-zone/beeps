mod conn;
mod response;

use axum::{http::header::AUTHORIZATION, response::IntoResponse, routing::get, Router};
use clap::Parser;
use conn::Conn;
use response::internal_error;
use sqlx::{postgres::PgPoolOptions, Postgres};
use std::{iter::once, time::Duration};
use tokio::net::TcpListener;
use tower_http::{compression, limit, sensitive_headers, timeout, trace};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Keep track of what you're doing throughout the day by being annoyed by a robot.
#[derive(Parser, Debug)]
#[clap(version)]
struct Options {
    #[clap(long, env, default_value = "info")]
    log_level: LevelFilter,

    #[clap(long, env, default_value = "0.0.0.0:3000")]
    address: String,

    /// Request body size limit, in bytes
    #[clap(long, env, default_value = "5242880")]
    body_limit: usize,

    /// Request timeout, in seconds
    #[clap(long, env, default_value = "5", value_parser = duration_parser)]
    request_timeout: Duration,

    #[clap(long, env)]
    database_url: String,

    #[clap(long, env, default_value = "3", value_parser = duration_parser)]
    database_acquire_timeout: Duration,

    #[clap(long, env, default_value = "5")]
    database_max_connections: u32,
}

fn duration_parser(s: &str) -> Result<Duration, std::num::ParseIntError> {
    s.parse().map(Duration::from_secs)
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    // TODO: opentelemetry
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(options.log_level.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(options.database_max_connections)
        .acquire_timeout(options.database_acquire_timeout)
        .connect(&options.database_url)
        .await
        .expect("can't connect to database");

    let app = Router::new()
        .layer(trace::TraceLayer::new_for_http())
        .layer(compression::CompressionLayer::new())
        .layer(limit::RequestBodyLimitLayer::new(options.body_limit))
        .layer(sensitive_headers::SetSensitiveHeadersLayer::new(once(
            AUTHORIZATION,
        )))
        .layer(timeout::TimeoutLayer::new(options.request_timeout))
        // ROUTES
        .route("/", get(hello_world))
        // STATE
        .with_state(pool);

    let listener = TcpListener::bind(options.address).await.unwrap();
    tracing::info!(address = ?listener.local_addr(), "listening");

    axum::serve(listener, app).await.unwrap();
}

async fn hello_world(Conn(mut conn): Conn) -> impl IntoResponse {
    sqlx::query_scalar::<Postgres, String>("select 'hello world from pg'")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)
}
