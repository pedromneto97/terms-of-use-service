mod cache;
mod database;
mod publisher;
mod storage;

// Database adapters
#[cfg(all(feature = "postgres", not(feature = "dynamodb")))]
pub use database::postgres::PostgresRepository as AppRepository;

#[cfg(all(feature = "dynamodb", not(feature = "postgres")))]
pub use database::dynamodb::DynamoRepository as AppRepository;

// Cache adapters
#[cfg(all(feature = "redis", not(feature = "valkey")))]
pub use cache::redis::RedisCache as Cache;

#[cfg(all(feature = "valkey", not(feature = "redis")))]
pub use cache::valkey::ValkeyCache as Cache;

#[cfg(not(any(feature = "redis", feature = "valkey")))]
pub use cache::noop::NoopCache as Cache;

// Storage adapters
#[cfg(all(feature = "s3", not(feature = "gcloud")))]
pub use storage::s3::S3Storage as Storage;

#[cfg(all(feature = "gcloud", not(feature = "s3")))]
pub use storage::gcloud::GoogleCloudStorage as Storage;

// Publisher adapters
#[cfg(feature = "sns")]
pub use publisher::sns::SNSPublisher as Publisher;

#[cfg(all(not(feature = "sns"), feature = "kafka"))]
pub use publisher::kafka::KafkaPublisher as Publisher;

#[cfg(not(any(feature = "sns", feature = "kafka")))]
pub use publisher::noop::NoopPublisher as Publisher;
