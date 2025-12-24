use std::time::Duration;

use tokio::time;
use tokio_stream::wrappers::WatchStream;
use tonic::{Request, Response, Status};
use tonic_health::{
    ServingStatus,
    pb::{self, HealthCheckRequest, HealthCheckResponse, health_server::Health},
};

use crate::grpc::server::GrpcService;

fn map_status_to_response(status: ServingStatus) -> HealthCheckResponse {
    HealthCheckResponse {
        status: pb::health_check_response::ServingStatus::from(status) as i32,
    }
}

#[tonic::async_trait]
impl Health for GrpcService {
    async fn check(
        &self,
        _: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let status = self.config.ping().await.values().all(|status| *status);

        let serving_status = if status {
            ServingStatus::Serving
        } else {
            ServingStatus::NotServing
        };

        Ok(Response::new(map_status_to_response(serving_status)))
    }

    type WatchStream = WatchStream<Result<HealthCheckResponse, Status>>;

    async fn watch(
        &self,
        _: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let (tx, rx) = tokio::sync::watch::channel(Ok(HealthCheckResponse {
            status: pb::health_check_response::ServingStatus::Unknown as i32,
        }));

        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                let status = config.ping().await.values().all(|status| *status);

                let serving_status = if status {
                    ServingStatus::Serving
                } else {
                    ServingStatus::NotServing
                };

                let response = Ok(map_status_to_response(serving_status));

                if tx.send(response).is_err() {
                    break;
                }

                time::sleep(Duration::from_secs(5)).await;
            }
        });

        Ok(Response::new(WatchStream::new(rx)))
    }
}
