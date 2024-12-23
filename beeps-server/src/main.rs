//! A sync server for beeps.

use clap::Parser;
use std::time::Duration;

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

    println!("{:#?}", options);
}
