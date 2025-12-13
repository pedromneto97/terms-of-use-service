use std::error::Error;

use dotenvy::dotenv;

mod core;
mod inbound;

#[cfg(all(
    feature = "dynamodb",
    feature = "postgres",
    not(any(test, clippy, rustfmt))
))]
compile_error!("Features 'dynamodb' and 'postgres' cannot be enabled at the same time.");

#[cfg(not(any(feature = "dynamodb", feature = "postgres", test, clippy, rustfmt)))]
compile_error!("Either feature 'dynamodb' or 'postgres' must be enabled.");

#[cfg(not(any(feature = "s3", feature = "gcloud", test, clippy, rustfmt)))]
compile_error!("No storage feature enabled. Please enable at least one: 's3' or 'gcloud'.");

#[cfg(all(feature = "s3", feature = "gcloud", not(any(test, clippy, rustfmt))))]
compile_error!("Multiple storage features enabled. Please enable only one: 's3' or 'gcloud'.");

#[cfg(all(feature = "redis", feature = "valkey", not(any(test, clippy, rustfmt))))]
compile_error!("Features 'redis' and 'valkey' cannot be enabled at the same time.");

#[tokio::main]
async fn main() -> Result<(), impl Error> {
    dotenv().ok();

    #[cfg(feature = "otel")]
    let _provider = core::telemetry::init_telemetry();

    let config = core::Config::new().await;

    inbound::start_server(config).await
}
