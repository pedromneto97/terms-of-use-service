use std::sync::Arc;

use domain::data::{
    repository::DatabaseRepository,
    service::{CacheService, PublisherService, StorageService},
};

#[derive(Clone)]
pub struct Config {
    pub repository: Arc<dyn DatabaseRepository>,
    pub cache: Arc<dyn CacheService>,
    pub storage: Arc<dyn StorageService>,
    pub publisher: Arc<dyn PublisherService>,
}

impl Config {
    pub async fn new(
        repository: Arc<dyn DatabaseRepository>,
        cache: Arc<dyn CacheService>,
        storage: Arc<dyn StorageService>,
        publisher: Arc<dyn PublisherService>,
    ) -> Self {
        Config {
            repository,
            cache,
            storage,
            publisher,
        }
    }
}
