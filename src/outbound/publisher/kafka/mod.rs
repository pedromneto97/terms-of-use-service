use chrono::Utc;
use domain::{data::service::PublisherService, dto::AcceptedTermOfUseDTO, errors::TermsOfUseError};
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

impl PublisherService for KafkaPublisher {
    #[tracing::instrument(skip(self, dto))]
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<(), TermsOfUseError> {
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
