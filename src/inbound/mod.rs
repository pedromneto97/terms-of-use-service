use std::error::Error;

use crate::core::Config;

#[cfg(feature = "actix-web")]
mod actix;

#[cfg(feature = "grpc")]
mod grpc;

// Compile-time validation: ensure only one server feature is enabled
#[cfg(all(feature = "actix-web", feature = "grpc"))]
compile_error!(
    "Features 'actix-web' and 'grpc' cannot be enabled simultaneously. Choose one server type."
);

#[cfg(not(any(feature = "actix-web", feature = "grpc")))]
compile_error!("No server feature enabled. Please enable at least one: 'actix-web' or 'grpc'.");

pub async fn start_server(config: Config) -> Result<(), impl Error> {
    #[cfg(feature = "actix-web")]
    return actix::start_server(config).await;

    #[cfg(feature = "grpc")]
    return grpc::start_server(config).await;
}
