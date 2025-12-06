use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web::Data,
};

use crate::core::Config;

mod error;
mod v1;

pub async fn start_server(config: Config) -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16 number");

    HttpServer::new(move || {
        let app = App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(Data::new(config.clone()))
            .configure(v1::controller::configure);

        #[cfg(feature = "otel")]
        let app = {
            use opentelemetry_instrumentation_actix_web::{RequestMetrics, RequestTracing};

            app.wrap(RequestTracing::new())
                .wrap(RequestMetrics::default())
        };

        app
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
