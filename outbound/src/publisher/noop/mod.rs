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
