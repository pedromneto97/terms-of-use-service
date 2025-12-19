use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web::Data,
};
use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

use crate::config::Config;

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
            .app_data(Data::new(config.clone()))
            .configure(healthcheck::configure)
            .configure(v1::controller::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
