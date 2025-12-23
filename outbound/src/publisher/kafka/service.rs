use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use domain::{
    data::service::PublisherService,
    dto::AcceptedTermOfUseDTO,
    errors::{Result, TermsOfUseError},
};
use rdkafka::producer::FutureRecord;
use tracing::{error, info};

use super::KafkaPublisher;

#[async_trait]
impl PublisherService for KafkaPublisher {
    #[tracing::instrument(skip(self))]
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<()> {
        let json = serde_json::to_string(&dto).map_err(|err| {
            error!("Failed to serialize AcceptedTermOfUseDTO: {err}");

            TermsOfUseError::InternalServerError
        })?;

        let key = format!("{}:{}:{}", dto.group, dto.term_id, dto.user_id);

        let record = FutureRecord::to(&self.topic)
            .payload(&json)
            .key(&key)
            .timestamp(Utc::now().timestamp_millis());

        self.producer
            .send(record, Duration::ZERO)
            .await
            .map_err(|(err, _)| {
                error!(
                    "Failed to publish agreement for user_id {}, term_id {}, group '{}': {err}",
                    dto.user_id, dto.term_id, dto.group
                );
                TermsOfUseError::InternalServerError
            })?;

        info!(
            "Successfully published agreement for user_id {}, term_id {}, group '{}'",
            dto.user_id, dto.term_id, dto.group
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::dto::AcceptedTermOfUseDTO;
    use rdkafka::{config::ClientConfig, producer::FutureProducer};

    fn create_test_producer() -> FutureProducer {
        ClientConfig::new()
            .set("bootstrap.servers", "invalid:9092")
            .set("message.timeout.ms", "100")
            .create()
            .expect("Failed to create test producer")
    }

    #[tokio::test]
    #[test_log::test]
    async fn publish_returns_internal_error_on_send_failure() {
        let publisher = KafkaPublisher {
            producer: create_test_producer(),
            topic: "test-topic".to_string(),
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

    #[tokio::test]
    #[test_log::test]
    async fn should_publish_agreement_successfully() {
        let publisher = KafkaPublisher::new().await;

        let dto = AcceptedTermOfUseDTO {
            term_id: 1,
            user_id: 2,
            group: "privacy-policy".to_string(),
        };

        publisher.publish_agreement(dto).await.unwrap();
    }
}
