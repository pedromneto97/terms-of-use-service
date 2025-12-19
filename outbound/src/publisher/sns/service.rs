use async_trait::async_trait;
use aws_sdk_sns::error::SdkError;
use domain::{
    data::service::PublisherService,
    dto::AcceptedTermOfUseDTO,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use crate::SNSPublisher;

#[async_trait]
impl PublisherService for SNSPublisher {
    #[tracing::instrument(skip(self, dto))]
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<()> {
        let json = serde_json::to_string(&dto).map_err(|err| {
            error!("Failed to serialize AcceptedTermOfUseDTO: {err}");

            TermsOfUseError::InternalServerError
        })?;

        self.client
            .publish()
            .topic_arn(&self.topic_arn)
            .message(json)
            .send()
            .await
            .map_err(|err| {
                match err {
                    SdkError::ServiceError(err) => {
                        error!(
                            "Failed to publish message to SNS topic '{}': {}",
                            &self.topic_arn,
                            err.into_err()
                        );
                    }
                    err => {
                        error!(
                            "Failed to publish message to SNS topic '{}': {err}",
                            &self.topic_arn
                        );
                    }
                }

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
    use domain::{dto::AcceptedTermOfUseDTO, errors::TermsOfUseError};

    fn dto() -> AcceptedTermOfUseDTO {
        AcceptedTermOfUseDTO {
            term_id: 1,
            user_id: 2,
            group: "privacy-policy".to_string(),
        }
    }

    #[tokio::test]
    #[test_log::test]
    async fn publish_returns_internal_error_on_send_failure() {
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

        let res = publisher.publish_agreement(dto()).await;

        let err = res.err().unwrap();

        assert!(matches!(err, TermsOfUseError::InternalServerError));
    }

    #[tokio::test]
    #[test_log::test]
    async fn should_publish_agreement_successfully() {
        let publisher = SNSPublisher::new().await;

        let account_id = std::env::var("AWS_ACCOUNT_ID").expect("AWS_ACCOUNT_ID must be set");
        let region = std::env::var("AWS_REGION").expect("AWS_REGION must be set");
        let topic_name = std::env::var("SNS_TOPIC_NAME").expect("SNS_TOPIC_NAME must be set");

        assert_eq!(
            publisher.topic_arn,
            format!("arn:aws:sns:{region}:{account_id}:{topic_name}")
        );

        publisher
            .client
            .create_topic()
            .name(&topic_name)
            .send()
            .await
            .unwrap();

        let res = publisher.publish_agreement(dto()).await;

        assert!(
            res.is_ok(),
            "Expected successful publish to LocalStack SNS, got: {:?}",
            res
        );
    }
}
