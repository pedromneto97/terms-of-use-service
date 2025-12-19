use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use super::DynamoRepository;

#[async_trait]
impl HealthCheck for DynamoRepository {
    async fn ping(&self) -> Result<()> {
        self.client
            .list_tables()
            .limit(1)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to ping DynamoDB database: {err}");

                TermsOfUseError::InternalServerError
            })
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use aws_config::BehaviorVersion;
    use domain::{data::health_check::HealthCheck, errors::TermsOfUseError};

    use super::DynamoRepository;

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_succeed_with_valid_connection() {
        let repo = DynamoRepository::new().await;

        let result = repo.ping().await;

        assert!(
            result.is_ok(),
            "ping should succeed with valid DynamoDB connection"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_returns_internal_server_error_on_failure() {
        // Create a repository with a client that has an invalid endpoint
        let config_builder = aws_config::defaults(BehaviorVersion::latest());
        let config = config_builder
            .endpoint_url("http://invalid:8000")
            .load()
            .await;
        let config_builder = aws_sdk_dynamodb::config::Builder::from(&config)
            .endpoint_url("http://invalid:8000")
            .build();

        let client = aws_sdk_dynamodb::Client::from_conf(config_builder);
        let repo = DynamoRepository { client };

        let result = repo.ping().await;

        let err = result.err().unwrap();
        assert!(
            matches!(err, TermsOfUseError::InternalServerError),
            "ping should return InternalServerError on failure"
        );
    }
}
