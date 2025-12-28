use actix_multipart::form::MultipartFormConfig;
use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web::{Data, JsonConfig, QueryConfig},
};
use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

use crate::{
    actix::error::{json_error_handler, multipart_error_handler, query_error_handler},
    config::Config,
};

mod error;
mod healthcheck;
mod v1;

pub async fn start_actix_server(config: Config) -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16 number");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(RequestTracing::new())
            .wrap(RequestMetrics::default())
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(QueryConfig::default().error_handler(query_error_handler))
            .app_data(MultipartFormConfig::default().error_handler(multipart_error_handler))
            .app_data(Data::new(config.clone()))
            .configure(healthcheck::configure)
            .configure(v1::controller::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
