use domain::{data::service::PublisherService, dto::AcceptedTermOfUseDTO, errors::TermsOfUseError};

#[derive(Clone, Debug)]
pub struct NoopPublisher;

impl NoopPublisher {
    pub async fn new() -> Self {
        Self
    }
}

impl PublisherService for NoopPublisher {
    async fn publish_agreement(&self, _: AcceptedTermOfUseDTO) -> Result<(), TermsOfUseError> {
        Ok(())
    }
}
