use std::error::Error;

use dotenvy::dotenv;

mod core;
pub(crate) mod domain;
mod inbound;
mod outbound;

#[tokio::main]
async fn main() -> Result<(), impl Error> {
    dotenv().ok();

    #[cfg(feature = "otel")]
    let _provider = core::telemetry::init_telemetry();

    let config = core::Config::new().await;

    inbound::start_server(config).await
}
