use async_trait::async_trait;
use chrono::Utc;
use domain::{
    data::service::PublisherService,
    dto::AcceptedTermOfUseDTO,
    errors::{Result, TermsOfUseError},
};
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;
use tracing::{error, info};

#[derive(Clone)]
pub struct KafkaPublisher {
    producer: FutureProducer,
    topic: String,
}

impl std::fmt::Debug for KafkaPublisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KafkaPublisher")
            .field("topic", &self.topic)
            .finish()
    }
}

impl KafkaPublisher {
    pub async fn new() -> Self {
        let brokers =
            std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let topic =
            std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "terms-of-use-agreements".to_string());

        info!("Initializing Kafka producer with brokers: {brokers}");

        let producer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.messages", "10000")
            .set("queue.buffering.max.kbytes", "1048576")
            .set("batch.num.messages", "100")
            .create::<FutureProducer>();
        if let Err(err) = producer {
            panic!("Failed to create Kafka producer: {err}");
        }
        let producer = producer.unwrap();

        Self { producer, topic }
    }
}

#[async_trait]
impl PublisherService for KafkaPublisher {
    #[tracing::instrument(skip(self, dto))]
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
    async fn new_builds_publisher_with_env_vars() {
        unsafe {
            std::env::set_var("KAFKA_BROKERS", "testbroker:9092");
            std::env::set_var("KAFKA_TOPIC", "test-topic");
        }

        let publisher = KafkaPublisher::new().await;

        assert_eq!(publisher.topic, "test-topic");

        unsafe {
            std::env::remove_var("KAFKA_BROKERS");
            std::env::remove_var("KAFKA_TOPIC");
        }
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
}
