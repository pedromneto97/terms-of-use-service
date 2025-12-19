use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use crate::GoogleCloudStorage;

#[async_trait]
impl HealthCheck for GoogleCloudStorage {
    async fn ping(&self) -> Result<()> {
        self.control_client
            .list_buckets()
            .set_page_size(1)
            .send()
            .await
            .map_err(|err| {
                error!(
                    "Failed to ping Google Cloud Storage bucket '{}': {err}",
                    &self.bucket_name
                );

                TermsOfUseError::InternalServerError
            })
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::GoogleCloudStorage;
    use domain::{data::health_check::HealthCheck, errors::TermsOfUseError};
    use google_cloud_storage::client::{Storage, StorageControl};

    async fn build_test_storage(bucket_name: &str) -> GoogleCloudStorage {
        let client = Storage::builder()
            .build()
            .await
            .expect("Failed to build test Storage client");
        let control_client = StorageControl::builder()
            .build()
            .await
            .expect("Failed to build test StorageControl client");

        GoogleCloudStorage {
            bucket: format!("projects/_/buckets/{bucket_name}"),
            bucket_name: bucket_name.to_string(),
            client,
            control_client,
        }
    }

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_returns_internal_error_on_failure() {
        let storage = build_test_storage("test-gcs-bucket").await;

        let result = storage.ping().await;

        let err = result.err().unwrap();

        assert!(matches!(err, TermsOfUseError::InternalServerError));
    }
}
