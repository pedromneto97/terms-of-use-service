use domain::data::PublisherServiceWithHealthCheck;
use rdkafka::{config::ClientConfig, producer::FutureProducer};

use tracing::info;

mod health_check;
mod service;

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

impl PublisherServiceWithHealthCheck for KafkaPublisher {}

#[cfg(test)]
mod tests {
    use super::KafkaPublisher;

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
}
