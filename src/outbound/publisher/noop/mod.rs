use crate::domain::{data::service::PublisherService, errors::TermsOfUseError};

#[derive(Clone, Debug)]
pub struct NoopPublisher;

impl NoopPublisher {
    pub async fn new() -> Self {
        Self
    }
}

impl PublisherService for NoopPublisher {
    async fn publish_agreement(
        &self,
        _user_id: i32,
        _term_id: i32,
        _group: &str,
    ) -> Result<(), TermsOfUseError> {
        Ok(())
    }
}
