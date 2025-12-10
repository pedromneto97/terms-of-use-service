use aws_config::BehaviorVersion;
use aws_sdk_sns::{Client, config::Builder};
use domain::{data::service::PublisherService, dto::AcceptedTermOfUseDTO, errors::TermsOfUseError};
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
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<(), TermsOfUseError> {
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
