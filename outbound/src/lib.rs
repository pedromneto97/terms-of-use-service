mod cache;
mod database;
mod publisher;
mod storage;

// Database adapters
#[cfg(feature = "postgres")]
pub use database::postgres::PostgresRepository;

#[cfg(feature = "dynamodb")]
pub use database::dynamodb::DynamoRepository;

// Cache adapters
#[cfg(feature = "redis")]
pub use cache::redis::RedisCache;

#[cfg(feature = "valkey")]
pub use cache::valkey::ValkeyCache;

#[cfg(any(not(feature = "cache"), clippy, rustfmt, test))]
pub use cache::noop::NoopCache;

// Storage adapters
#[cfg(feature = "s3")]
pub use storage::s3::S3Storage;

#[cfg(feature = "gcloud")]
pub use storage::gcloud::GoogleCloudStorage;

// Publisher adapters
#[cfg(feature = "sns")]
pub use publisher::sns::SNSPublisher;

#[cfg(feature = "kafka")]
pub use publisher::kafka::KafkaPublisher;

#[cfg(any(not(feature = "publisher"), clippy, rustfmt, test))]
pub use publisher::noop::NoopPublisher;
