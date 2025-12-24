use std::sync::Arc;

use crate::{
    config::Config,
    mocks::{MockCacheService, MockDatabaseRepository, MockPublisherService, MockStorageService},
};

mod create_consent_test;
mod create_term_test;
mod get_latest_terms_test;
mod has_consent_test;
mod health_check_test;

pub fn create_test_config(
    repository: Option<MockDatabaseRepository>,
    cache: Option<MockCacheService>,
    storage: Option<MockStorageService>,
    publisher: Option<MockPublisherService>,
) -> Arc<Config> {
    Arc::new(Config {
        repository: Arc::new(repository.unwrap_or(MockDatabaseRepository::new())),
        cache: Arc::new(cache.unwrap_or(MockCacheService::new())),
        storage: Arc::new(storage.unwrap_or(MockStorageService::new())),
        publisher: Arc::new(publisher.unwrap_or(MockPublisherService::new())),
    })
}
