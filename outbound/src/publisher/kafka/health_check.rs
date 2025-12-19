use std::time::Duration;

use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use rdkafka::producer::Producer;
use tracing::error;

use super::KafkaPublisher;

#[async_trait]
impl HealthCheck for KafkaPublisher {
    async fn ping(&self) -> Result<()> {
        match self
            .producer
            .client()
            .fetch_metadata(None, Duration::from_secs(3))
        {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Failed to ping Kafka service: {err}");

                Err(TermsOfUseError::InternalServerError)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use domain::{data::health_check::HealthCheck, errors::TermsOfUseError};
    use rdkafka::{ClientConfig, producer::FutureProducer};

    use super::KafkaPublisher;

    fn create_test_producer() -> FutureProducer {
        ClientConfig::new()
            .set("bootstrap.servers", "invalid:9092")
            .set("message.timeout.ms", "100")
            .create()
            .expect("Failed to create test producer")
    }

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_fail_with_invalid_broker() {
        let publisher = KafkaPublisher {
            producer: create_test_producer(),
            topic: "test-topic".to_string(),
        };

        let result = publisher.ping().await;

        assert!(
            result.is_err(),
            "ping should fail with invalid Kafka broker"
        );
        assert!(matches!(
            result.err().unwrap(),
            TermsOfUseError::InternalServerError
        ));
    }

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_returns_internal_error_on_metadata_fetch_failure() {
        // Create a producer with an invalid bootstrap server
        let producer = ClientConfig::new()
            .set("bootstrap.servers", "invalid-broker:9092")
            .set("message.timeout.ms", "100")
            .create::<FutureProducer>()
            .expect("Failed to create producer");

        let publisher = KafkaPublisher {
            producer,
            topic: "test-topic".to_string(),
        };

        let result = publisher.ping().await;

        assert!(
            result.is_err(),
            "ping should return error for invalid broker"
        );
        assert!(matches!(
            result.err().unwrap(),
            TermsOfUseError::InternalServerError
        ));
    }
}
