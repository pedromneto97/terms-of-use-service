use async_trait::async_trait;

use crate::{dto::AcceptedTermOfUseDTO, errors::Result};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PublisherService: Send + Sync {
    async fn publish_agreement(&self, dto: AcceptedTermOfUseDTO) -> Result<()>;
}
