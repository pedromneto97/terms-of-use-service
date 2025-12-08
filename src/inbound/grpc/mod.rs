use std::error::Error;

use tonic::transport::Server;
#[cfg(feature = "otel")]
use tonic_tracing_opentelemetry::middleware::server::OtelGrpcLayer;
use tracing::error;

use crate::{
    core::Config,
    inbound::grpc::{server::GrpcService, terms_of_use_service_server::TermsOfUseServiceServer},
};

tonic::include_proto!("terms_of_use");

mod file_upload;
mod mapper;
mod server;

pub async fn start_server(config: Config) -> Result<(), impl Error> {
    let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .expect("GRPC_PORT must be a valid u16 number");

    let addr = format!("{host}:{port}").parse().expect("Invalid host/port");
    let service = GrpcService::new(config);

    let mut server = Server::builder();
    #[cfg(feature = "otel")]
    let mut server = server.layer(OtelGrpcLayer::default());

    server
        .add_service(TermsOfUseServiceServer::new(service))
        .serve(addr)
        .await
        .map_err(|e| {
            error!("gRPC server error: {e}");
            e
        })
}
