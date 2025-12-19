use std::{collections::HashMap, sync::Arc};

use domain::data::{
    CacheServiceWithHealthCheck, DatabaseRepositoryWithHealthCheck,
    PublisherServiceWithHealthCheck, StorageServiceWithHealthCheck,
};

#[derive(Clone)]
pub struct Config {
    pub repository: Arc<dyn DatabaseRepositoryWithHealthCheck>,
    pub cache: Arc<dyn CacheServiceWithHealthCheck>,
    pub storage: Arc<dyn StorageServiceWithHealthCheck>,
    pub publisher: Arc<dyn PublisherServiceWithHealthCheck>,
}

impl Config {
    pub async fn new(
        repository: Arc<dyn DatabaseRepositoryWithHealthCheck>,
        cache: Arc<dyn CacheServiceWithHealthCheck>,
        storage: Arc<dyn StorageServiceWithHealthCheck>,
        publisher: Arc<dyn PublisherServiceWithHealthCheck>,
    ) -> Self {
        Config {
            repository,
            cache,
            storage,
            publisher,
        }
    }

    pub async fn ping(&self) -> HashMap<&'static str, bool> {
        let mut results = HashMap::new();

        let (repository, cache, storage, publisher) = futures::future::join4(
            self.repository.ping(),
            self.cache.ping(),
            self.storage.ping(),
            self.publisher.ping(),
        )
        .await;

        results.insert("repository", repository.is_ok());
        results.insert("cache", cache.is_ok());
        results.insert("storage", storage.is_ok());
        results.insert("publisher", publisher.is_ok());

        results
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use domain::errors::TermsOfUseError;

    use crate::{Config, mocks::*};

    #[cfg(feature = "actix-web")]
    #[tokio::test]
    async fn config_new_creates_instance_with_services() {
        let repository = MockDatabaseRepository::new();
        let cache = MockCacheService::new();
        let storage = MockStorageService::new();
        let publisher = MockPublisherService::new();

        let config = Config::new(
            Arc::new(repository),
            Arc::new(cache),
            Arc::new(storage),
            Arc::new(publisher),
        )
        .await;

        // Verify that the config was created successfully
        assert!(Arc::strong_count(&config.repository) >= 1);
        assert!(Arc::strong_count(&config.cache) >= 1);
        assert!(Arc::strong_count(&config.storage) >= 1);
        assert!(Arc::strong_count(&config.publisher) >= 1);
    }

    #[cfg(feature = "actix-web")]
    #[tokio::test]
    async fn ping_returns_all_healthy_when_services_are_ok() {
        let mut repository = MockDatabaseRepository::new();
        repository.expect_ping().returning(|| Ok(()));

        let mut cache = MockCacheService::new();
        cache.expect_ping().returning(|| Ok(()));

        let mut storage = MockStorageService::new();
        storage.expect_ping().returning(|| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let config = Config::new(
            Arc::new(repository),
            Arc::new(cache),
            Arc::new(storage),
            Arc::new(publisher),
        )
        .await;

        let results = config.ping().await;

        assert_eq!(results.get("repository"), Some(&true));
        assert_eq!(results.get("cache"), Some(&true));
        assert_eq!(results.get("storage"), Some(&true));
        assert_eq!(results.get("publisher"), Some(&true));
        assert_eq!(results.len(), 4);
    }

    #[cfg(feature = "actix-web")]
    #[tokio::test]
    async fn ping_returns_false_for_unhealthy_repository() {
        let mut repository = MockDatabaseRepository::new();
        repository
            .expect_ping()
            .returning(|| Err(TermsOfUseError::InternalServerError));

        let mut cache = MockCacheService::new();
        cache.expect_ping().returning(|| Ok(()));

        let mut storage = MockStorageService::new();
        storage.expect_ping().returning(|| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let config = Config::new(
            Arc::new(repository),
            Arc::new(cache),
            Arc::new(storage),
            Arc::new(publisher),
        )
        .await;

        let results = config.ping().await;

        assert_eq!(results.get("repository"), Some(&false));
        assert_eq!(results.get("cache"), Some(&true));
        assert_eq!(results.get("storage"), Some(&true));
        assert_eq!(results.get("publisher"), Some(&true));
    }

    #[cfg(feature = "actix-web")]
    #[tokio::test]
    async fn ping_returns_false_for_unhealthy_cache() {
        let mut repository = MockDatabaseRepository::new();
        repository.expect_ping().returning(|| Ok(()));

        let mut cache = MockCacheService::new();
        cache
            .expect_ping()
            .returning(|| Err(TermsOfUseError::InternalServerError));

        let mut storage = MockStorageService::new();
        storage.expect_ping().returning(|| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let config = Config::new(
            Arc::new(repository),
            Arc::new(cache),
            Arc::new(storage),
            Arc::new(publisher),
        )
        .await;

        let results = config.ping().await;

        assert_eq!(results.get("repository"), Some(&true));
        assert_eq!(results.get("cache"), Some(&false));
        assert_eq!(results.get("storage"), Some(&true));
        assert_eq!(results.get("publisher"), Some(&true));
    }

    #[cfg(feature = "actix-web")]
    #[tokio::test]
    async fn ping_returns_multiple_false_when_multiple_services_fail() {
        let mut repository = MockDatabaseRepository::new();
        repository
            .expect_ping()
            .returning(|| Err(TermsOfUseError::InternalServerError));

        let mut cache = MockCacheService::new();
        cache.expect_ping().returning(|| Ok(()));

        let mut storage = MockStorageService::new();
        storage
            .expect_ping()
            .returning(|| Err(TermsOfUseError::NotFound));

        let mut publisher = MockPublisherService::new();
        publisher.expect_ping().returning(|| Ok(()));

        let config = Config::new(
            Arc::new(repository),
            Arc::new(cache),
            Arc::new(storage),
            Arc::new(publisher),
        )
        .await;

        let results = config.ping().await;

        assert_eq!(results.get("repository"), Some(&false));
        assert_eq!(results.get("cache"), Some(&true));
        assert_eq!(results.get("storage"), Some(&false));
        assert_eq!(results.get("publisher"), Some(&true));
    }

    #[cfg(feature = "actix-web")]
    #[tokio::test]
    async fn config_is_cloneable() {
        let repository = MockDatabaseRepository::new();
        let cache = MockCacheService::new();
        let storage = MockStorageService::new();
        let publisher = MockPublisherService::new();

        let config = Config::new(
            Arc::new(repository),
            Arc::new(cache),
            Arc::new(storage),
            Arc::new(publisher),
        )
        .await;

        let cloned_config = config.clone();

        // Verify that Arc counts increased after cloning
        assert!(Arc::strong_count(&config.repository) >= 2);
        assert!(Arc::strong_count(&cloned_config.repository) >= 2);
    }
}
