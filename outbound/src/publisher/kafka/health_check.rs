use std::time::Duration;

use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use rdkafka::producer::Producer;
use tokio::task;
use tracing::error;

use super::KafkaPublisher;

#[async_trait]
impl HealthCheck for KafkaPublisher {
    async fn ping(&self) -> Result<()> {
        let producer = self.producer.clone();
        match task::spawn_blocking(move || {
            producer
                .client()
                .fetch_metadata(None, Duration::from_millis(500))
        })
        .await
        {
            Ok(metadata) => match metadata {
                Ok(metadata) => {
                    if metadata.brokers().is_empty() {
                        error!("No brokers found in Kafka metadata");

                        Err(TermsOfUseError::InternalServerError)
                    } else {
                        Ok(())
                    }
                }
                Err(err) => {
                    error!("Failed to fetch Kafka metadata: {err}");

                    Err(TermsOfUseError::InternalServerError)
                }
            },
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
    use rdkafka::ClientConfig;

    use super::KafkaPublisher;

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_fail_with_invalid_broker() {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", "invalid:9092")
            .set("message.timeout.ms", "100")
            .create()
            .expect("Failed to create test producer");

        let publisher = KafkaPublisher {
            producer,
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
    async fn health_check_ping_returns_ok() {
        let publisher = KafkaPublisher::new().await;

        let result = publisher.ping().await;

        assert!(
            result.is_ok(),
            "ping should succeed with valid Kafka broker"
        );
    }
}
