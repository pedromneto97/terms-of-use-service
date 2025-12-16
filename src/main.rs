use std::{error::Error, sync::Arc};

use domain::data::{
    repository::DatabaseRepository,
    service::{CacheService, PublisherService, StorageService},
};
use dotenvy::dotenv;

use crate::core::Config;

mod core;
mod inbound;

#[cfg(all(
    feature = "dynamodb",
    feature = "postgres",
    not(any(test, clippy, rustfmt))
))]
compile_error!("Features 'dynamodb' and 'postgres' cannot be enabled at the same time.");

#[cfg(not(any(feature = "dynamodb", feature = "postgres", test, clippy, rustfmt)))]
compile_error!("Either feature 'dynamodb' or 'postgres' must be enabled.");

#[cfg(not(any(feature = "s3", feature = "gcloud", test, clippy, rustfmt)))]
compile_error!("No storage feature enabled. Please enable at least one: 's3' or 'gcloud'.");

#[cfg(all(feature = "s3", feature = "gcloud", not(any(test, clippy, rustfmt))))]
compile_error!("Multiple storage features enabled. Please enable only one: 's3' or 'gcloud'.");

#[cfg(all(feature = "redis", feature = "valkey", not(any(test, clippy, rustfmt))))]
compile_error!("Features 'redis' and 'valkey' cannot be enabled at the same time.");

async fn get_repository() -> Arc<dyn DatabaseRepository> {
    #[cfg(feature = "dynamodb")]
    return Arc::new(outbound::DynamoRepository::new().await);

    #[cfg(feature = "postgres")]
    return Arc::new(outbound::PostgresRepository::new().await);

    unreachable!("No database feature (postgres or dynamodb) enabled")
}

async fn get_cache() -> impl CacheService {
    #[cfg(feature = "redis")]
    return outbound::RedisCache::new().await;
    #[cfg(feature = "valkey")]
    return outbound::ValkeyCache::new().await;
    #[cfg(not(any(feature = "redis", feature = "valkey")))]
    return outbound::NoopCache::new().await;
}

async fn get_publisher() -> impl PublisherService {
    #[cfg(feature = "sns")]
    return outbound::SNSPublisher::new().await;
    #[cfg(feature = "kafka")]
    return outbound::KafkaPublisher::new().await;
    #[cfg(not(any(feature = "sns", feature = "kafka")))]
    return outbound::NoopPublisher::new().await;
}

async fn get_storage() -> Arc<dyn StorageService> {
    #[cfg(feature = "s3")]
    return Arc::new(outbound::S3Storage::new().await);
    #[cfg(feature = "gcloud")]
    return Arc::new(outbound::GoogleCloudStorage::new().await);

    unreachable!("No storage feature (s3 or gcloud) enabled")
}

#[tokio::main]
async fn main() -> Result<(), impl Error> {
    dotenv().ok();

    #[cfg(feature = "otel")]
    let _provider = core::telemetry::init_telemetry();

    let repository = get_repository().await;
    let cache = get_cache().await;
    let storage = get_storage().await;
    let publisher = get_publisher().await;

    let config = Config::new(repository, Arc::new(cache), storage, Arc::new(publisher)).await;

    inbound::start_server(config).await
}
