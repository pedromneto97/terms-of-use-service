use domain::data::{
    CacheServiceWithHealthCheck, DatabaseRepositoryWithHealthCheck,
    PublisherServiceWithHealthCheck, StorageServiceWithHealthCheck,
    health_check::HealthCheck,
    repository::{
        DatabaseRepository as DatabaseRepositoryTrait, TermRepository, UserAgreementRepository,
    },
    service::{CacheService, PublisherService, StorageService},
};
use domain::errors::Result;
use mockall::mock;
use std::path::Path;

mock! {
    pub DatabaseRepository {}

    #[async_trait::async_trait]
    impl TermRepository for DatabaseRepository {
        async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<domain::entities::TermOfUse>>;
        async fn get_term_by_id(&self, term_id: i32) -> Result<Option<domain::entities::TermOfUse>>;
        async fn create_term(&self, term: domain::entities::TermOfUse) -> Result<domain::entities::TermOfUse>;
    }

    #[async_trait::async_trait]
    impl UserAgreementRepository for DatabaseRepository {
        async fn has_user_agreed_to_term(&self, user_id: i32, term_id: i32) -> Result<bool>;
        async fn create_user_agreement(&self, user_id: i32, term_id: i32) -> Result<()>;
    }

    #[async_trait::async_trait]
    impl HealthCheck for DatabaseRepository {
        async fn ping(&self) -> Result<()>;
    }

    impl DatabaseRepositoryTrait for DatabaseRepository { }
}

impl DatabaseRepositoryWithHealthCheck for MockDatabaseRepository {}

mock! {
    pub CacheService {}

    #[async_trait::async_trait]
    impl CacheService for CacheService {
        async fn find_user_agreement(&self, user_id: i32, group: &str) -> Result<Option<bool>>;

        async fn store_user_agreement(&self, user_id: i32, group: &str, agreed: bool) -> Result<()>;

        async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<domain::entities::TermOfUse>>;

        async fn store_latest_term_for_group(&self, term: &domain::entities::TermOfUse) -> Result<()>;

        async fn invalidate_cache_for_group(&self, group: &str) -> Result<()>;
    }

    #[async_trait::async_trait]
    impl HealthCheck for CacheService {
        async fn ping(&self) -> Result<()>;
    }
}

impl CacheServiceWithHealthCheck for MockCacheService {}

mock! {
    pub StorageService {}

    #[async_trait::async_trait]
    impl StorageService for StorageService {
        async fn upload_file(&self, file: &Path, content_type: &str) -> Result<String>;

        async fn delete_file(&self, path: &str) -> Result<()>;

        async fn get_file_url(&self, path: &str) -> Result<String>;
    }

    #[async_trait::async_trait]
    impl HealthCheck for StorageService {
        async fn ping(&self) -> Result<()>;
    }
}

impl StorageServiceWithHealthCheck for MockStorageService {}

mock! {
    pub PublisherService {}

    #[async_trait::async_trait]
    impl PublisherService for PublisherService {
        async fn publish_agreement(&self, dto: domain::dto::AcceptedTermOfUseDTO) -> Result<()>;
    }

    #[async_trait::async_trait]
    impl HealthCheck for PublisherService {
        async fn ping(&self) -> Result<()>;
    }
}

impl PublisherServiceWithHealthCheck for MockPublisherService {}
