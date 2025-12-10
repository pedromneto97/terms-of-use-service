use crate::{dto::AcceptedTermOfUseDTO, errors::Result};

#[cfg_attr(test, mockall::automock)]
pub trait PublisherService: Send + Sync {
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<()>;
}
