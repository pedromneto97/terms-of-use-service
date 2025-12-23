use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use super::S3Storage;

#[async_trait]
impl HealthCheck for S3Storage {
    async fn ping(&self) -> Result<()> {
        self.client
            .list_buckets()
            .max_buckets(1)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to ping S3 storage: {err}");

                TermsOfUseError::InternalServerError
            })
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use aws_config::{BehaviorVersion, Region};
    use aws_credential_types::Credentials;
    use aws_sdk_s3::config::SharedCredentialsProvider;
    use domain::{data::health_check::HealthCheck, errors::TermsOfUseError};

    use crate::S3Storage;

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_succeed_with_valid_s3_client() {
        let storage = S3Storage::new().await;

        let result = storage.ping().await;

        assert!(result.is_ok(), "ping should succeed with valid S3 client");
    }

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_returns_internal_error_on_failure() {
        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
            .region(Region::new("us-east-1"))
            .endpoint_url("http://invalid:4566")
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        let storage = S3Storage {
            bucket_name: "test-bucket".to_string(),
            client,
            endpoint_url: None,
        };

        let result = storage.ping().await;

        assert!(result.is_err(), "ping should fail with invalid S3 endpoint");
        assert!(matches!(
            result.err().unwrap(),
            TermsOfUseError::InternalServerError
        ));
    }
}
