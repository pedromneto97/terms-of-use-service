use std::collections::HashMap;

use actix_web::{HttpResponse, get, web::Data};
use serde::Serialize;

use crate::Config;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub services: HashMap<&'static str, bool>,
}

#[get("/health")]
#[tracing::instrument(skip(config))]
async fn health(config: Data<Config>) -> HttpResponse {
    let ping = config.ping().await;

    if ping.iter().any(|(_, v)| !*v) {
        HttpResponse::ServiceUnavailable().json(HealthResponse {
            status: "UNAVAILABLE".to_string(),
            services: ping,
        })
    } else {
        HttpResponse::Ok().json(HealthResponse {
            status: "OK".to_string(),
            services: ping,
        })
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(health);
}

#[cfg(test)]
mod tests {
    use actix_web::{App, http::StatusCode, test, web::Data};
    use serde_json::Value;
    use std::sync::Arc;

    use crate::{actix::healthcheck::configure, mocks::*};

    fn build_config(
        repository: MockDatabaseRepository,
        cache: MockCacheService,
        storage: MockStorageService,
        publisher: MockPublisherService,
    ) -> crate::Config {
        crate::Config {
            repository: Arc::new(repository),
            cache: Arc::new(cache),
            storage: Arc::new(storage),
            publisher: Arc::new(publisher),
        }
    }

    #[actix_web::test]
    async fn health_returns_ok_when_all_services_healthy() {
        let mut repository = MockDatabaseRepository::new();
        repository.expect_ping().returning(|| Ok(()));

        let mut cache = MockCacheService::new();
        cache.expect_ping().returning(|| Ok(()));

        let mut storage = MockStorageService::new();
        storage.expect_ping().returning(|| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(Data::new(build_config(
                    repository, cache, storage, publisher,
                )))
                .configure(configure),
        )
        .await;

        let response =
            test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = test::read_body(response).await;
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "OK");
        assert!(payload["services"]["repository"].is_boolean());
        assert!(payload["services"]["cache"].is_boolean());
        assert!(payload["services"]["storage"].is_boolean());
        assert!(payload["services"]["publisher"].is_boolean());
    }

    #[actix_web::test]
    async fn health_returns_service_unavailable_when_repository_unhealthy() {
        let mut repository = MockDatabaseRepository::new();
        repository
            .expect_ping()
            .returning(|| Err(domain::errors::TermsOfUseError::InternalServerError));

        let mut cache = MockCacheService::new();
        cache.expect_ping().returning(|| Ok(()));

        let mut storage = MockStorageService::new();
        storage.expect_ping().returning(|| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(Data::new(build_config(
                    repository, cache, storage, publisher,
                )))
                .configure(configure),
        )
        .await;

        let response =
            test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = test::read_body(response).await;
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "UNAVAILABLE");
        assert_eq!(payload["services"]["repository"], false);
    }

    #[actix_web::test]
    async fn health_returns_service_unavailable_when_cache_unhealthy() {
        let mut repository = MockDatabaseRepository::new();
        repository.expect_ping().returning(|| Ok(()));

        let mut cache = MockCacheService::new();
        cache
            .expect_ping()
            .returning(|| Err(domain::errors::TermsOfUseError::InternalServerError));

        let mut storage = MockStorageService::new();
        storage.expect_ping().returning(|| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(Data::new(build_config(
                    repository, cache, storage, publisher,
                )))
                .configure(configure),
        )
        .await;

        let response =
            test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = test::read_body(response).await;
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "UNAVAILABLE");
        assert_eq!(payload["services"]["cache"], false);
    }
}
