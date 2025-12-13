mod cache;
mod database;
mod publisher;
mod storage;

// Prevent incompatible or missing feature selections at compile time.
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

// Feature-aware re-exports so consumers can import adapters directly from `outbound`.

// Database adapters
// Database adapters (mutually exclusive; prefer Postgres under tests if both are enabled via `--all-features`).
#[cfg(all(feature = "postgres", not(feature = "dynamodb")))]
pub use database::postgres::PostgresRepository as AppRepository;

#[cfg(all(feature = "dynamodb", not(feature = "postgres")))]
pub use database::dynamodb::DynamoRepository as AppRepository;

// Cache adapters
// Cache adapters (mutually exclusive: `redis` or `valkey`; default to `noop`)
#[cfg(all(feature = "redis", feature = "valkey", not(any(test, clippy, rustfmt))))]
compile_error!("Features 'redis' and 'valkey' cannot be enabled at the same time.");

#[cfg(all(feature = "redis", not(feature = "valkey")))]
pub use cache::redis::RedisCache as Cache;

#[cfg(all(feature = "valkey", not(feature = "redis")))]
pub use cache::valkey::ValkeyCache as Cache;

#[cfg(not(any(feature = "redis", feature = "valkey")))]
pub use cache::noop::NoopCache as Cache;

// Storage adapters
// Storage adapters (mutually exclusive; default to S3 under tests if both are enabled via `--all-features`).
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
