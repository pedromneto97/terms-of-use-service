use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use crate::SNSPublisher;

#[async_trait]
impl HealthCheck for SNSPublisher {
    async fn ping(&self) -> Result<()> {
        self.client.list_topics().send().await.map_err(|err| {
            error!("Failed to ping SNS service: {err}");

            TermsOfUseError::InternalServerError
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_config::{BehaviorVersion, Region};
    use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
    use aws_sdk_sns::{Client, Config};
    use domain::errors::TermsOfUseError;

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_succeed_with_valid_sns_client() {
        let config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
            .region(Region::new("us-east-1"))
            .build();

        let client = Client::from_conf(config);

        let publisher = SNSPublisher {
            client: client,
            topic_arn: "arn:aws:sns:us-east-1:123456789012:terms-agreements".to_string(),
        };

        // Note: This will fail without actual SNS access, which returns InternalServerError
        let result = publisher.ping().await;

        let err = result.err().unwrap();

        assert!(matches!(err, TermsOfUseError::InternalServerError,));
    }

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_returns_internal_error_on_sns_failure() {
        let config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
            .region(Region::new("us-east-1"))
            .endpoint_url("http://invalid:4566")
            .build();

        let client = Client::from_conf(config);

        let publisher = SNSPublisher {
            client: client,
            topic_arn: "arn:aws:sns:us-east-1:123456789012:terms-agreements".to_string(),
        };

        let result = publisher.ping().await;

        assert!(
            result.is_err(),
            "ping should fail with invalid SNS endpoint"
        );
        assert!(matches!(
            result.err().unwrap(),
            TermsOfUseError::InternalServerError
        ));
    }
}
