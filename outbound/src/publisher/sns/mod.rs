use aws_config::BehaviorVersion;
use aws_sdk_sns::{Client, config::Builder};
use domain::{
    data::service::PublisherService,
    dto::AcceptedTermOfUseDTO,
    errors::{Result, TermsOfUseError},
};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct SNSPublisher {
    client: Client,
    topic_arn: String,
}

impl SNSPublisher {
    pub async fn new() -> Self {
        let config_builder = aws_config::defaults(BehaviorVersion::latest());
        let account_id = std::env::var("AWS_ACCOUNT_ID").expect("AWS_ACCOUNT_ID must be set");
        let region = std::env::var("AWS_REGION").expect("AWS_REGION must be set");
        let topic_name = std::env::var("SNS_TOPIC_NAME").expect("SNS_TOPIC_NAME must be set");

        let endpoint_url = std::env::var("AWS_ENDPOINT_URL").ok();

        let config = if let Some(ref url) = endpoint_url {
            info!("Using custom SNS endpoint URL: {url}");

            config_builder.endpoint_url(url).load().await
        } else {
            config_builder.load().await
        };

        let config_builder = Builder::from(&config).build();

        let client = Client::from_conf(config_builder);

        Self {
            client,
            topic_arn: format!("arn:aws:sns:{region}:{account_id}:{topic_name}"),
        }
    }
}

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
                error!(
                    "Failed to publish agreement for user_id {}, term_id {}, group '{}': {err}",
                    dto.user_id, dto.term_id, dto.group
                );

                TermsOfUseError::InternalServerError
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_config::Region;
    use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
    use aws_sdk_sns::Config;
    use domain::dto::AcceptedTermOfUseDTO;
    use domain::errors::TermsOfUseError;

    fn client() -> Client {
        let config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
            .region(Region::new("us-east-1"))
            .build();

        Client::from_conf(config)
    }

    #[tokio::test]
    async fn new_builds_topic_arn_from_env() {
        unsafe {
            std::env::set_var("AWS_ACCOUNT_ID", "123456789012");
            std::env::set_var("AWS_REGION", "us-east-1");
            std::env::set_var("SNS_TOPIC_NAME", "terms-agreements");
        }

        let publisher = SNSPublisher::new().await;

        assert_eq!(
            publisher.topic_arn,
            "arn:aws:sns:us-east-1:123456789012:terms-agreements"
        );

        unsafe {
            std::env::remove_var("AWS_ACCOUNT_ID");
            std::env::remove_var("AWS_REGION");
            std::env::remove_var("SNS_TOPIC_NAME");
        }
    }

    #[tokio::test]
    async fn publish_returns_internal_error_on_send_failure() {
        let publisher = SNSPublisher {
            client: client(),
            topic_arn: "arn:aws:sns:us-east-1:123456789012:terms-agreements".to_string(),
        };

        let dto = AcceptedTermOfUseDTO {
            term_id: 1,
            user_id: 2,
            group: "privacy-policy".to_string(),
        };

        let res = publisher.publish_agreement(dto).await;

        assert!(res.is_err());

        match res.err().unwrap() {
            TermsOfUseError::InternalServerError => {}
            other => panic!("expected InternalServerError, got {:?}", other),
        }
    }
}
