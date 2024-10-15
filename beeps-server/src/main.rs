use axum::{routing::get, Router};
use clap::Parser;
use tracing::level_filters::LevelFilter;

/// Keep track of what you're doing throughout the day by being annoyed by a robot.
#[derive(Parser, Debug)]
struct Options {
    #[clap(long = "log-level", default_value = "info")]
    log_level: LevelFilter,

    #[clap(long = "address", default_value = "0.0.0.0:3000")]
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
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::compression::CompressionLayer::new())
        .route("/", get(|| async { "Hello, World!" }));

    tracing::info!(address = &options.address, "listening");
    let listener = tokio::net::TcpListener::bind(options.address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
