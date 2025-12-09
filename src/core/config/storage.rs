#[cfg(feature = "s3")]
pub type Storage = crate::outbound::storage::s3::StorageConfig;

#[cfg(feature = "gcloud")]
pub type Storage = crate::outbound::storage::gcloud::GoogleCloudStorage;
