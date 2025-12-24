use std::{error::Error, sync::Arc};

use tonic::transport::Server;
use tonic_health::pb::health_server::HealthServer;
use tonic_tracing_opentelemetry::middleware::server::OtelGrpcLayer;
use tracing::error;

use crate::{
    Config,
    grpc::{server::GrpcService, terms_of_use_service_server::TermsOfUseServiceServer},
};

tonic::include_proto!("terms_of_use");

mod file_upload;
mod health_check;
mod mapper;
mod server;

#[cfg(test)]
mod tests;

pub async fn start_grpc_server(config: Config) -> Result<(), impl Error> {
    let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .expect("GRPC_PORT must be a valid u16 number");

    let addr = format!("{host}:{port}").parse().expect("Invalid host/port");
    let service = Arc::new(GrpcService::new(Arc::new(config)));

    let mut server = Server::builder().layer(OtelGrpcLayer::default());

    server
        .add_service(HealthServer::from_arc(service.clone()))
        .add_service(TermsOfUseServiceServer::from_arc(service))
        .serve(addr)
        .await
        .map_err(|e| {
            error!("gRPC server error: {e}");
            e
        })
}
