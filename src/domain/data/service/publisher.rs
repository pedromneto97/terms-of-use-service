use crate::domain::{dto::AcceptedTermOfUseDTO, errors::TermsOfUseError};

pub trait PublisherService: Send + Sync {
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<(), TermsOfUseError>;
}
