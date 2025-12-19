use async_trait::async_trait;
use domain::{
    data::{PublisherServiceWithHealthCheck, health_check::HealthCheck, service::PublisherService},
    dto::AcceptedTermOfUseDTO,
    errors::Result,
};

#[derive(Clone, Debug)]
pub struct NoopPublisher;

impl NoopPublisher {
    pub async fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PublisherService for NoopPublisher {
    async fn publish_agreement(&self, _: AcceptedTermOfUseDTO) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl HealthCheck for NoopPublisher {
    async fn ping(&self) -> Result<()> {
        Ok(())
    }
}

impl PublisherServiceWithHealthCheck for NoopPublisher {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_ping_should_always_succeed() {
        let publisher = NoopPublisher::new().await;

        let result = publisher.ping().await;

        assert!(
            result.is_ok(),
            "NoopPublisher.ping() should always return Ok(())"
        );
    }

    #[tokio::test]
    async fn publish_agreement_should_always_succeed() {
        let publisher = NoopPublisher::new().await;

        let dto = AcceptedTermOfUseDTO {
            term_id: 1,
            user_id: 2,
            group: "privacy-policy".to_string(),
        };

        let result = publisher.publish_agreement(dto).await;

        assert!(
            result.is_ok(),
            "NoopPublisher.publish_agreement() should always return Ok(())"
        );
    }
}
