use async_trait::async_trait;

use crate::errors::Result;

#[async_trait]
pub trait HealthCheck {
    async fn ping(&self) -> Result<()>;
}
