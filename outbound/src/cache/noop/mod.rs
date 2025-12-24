use async_trait::async_trait;
use domain::{
    data::{CacheServiceWithHealthCheck, health_check::HealthCheck, service::CacheService},
    entities::TermOfUse,
    errors::Result,
};

#[derive(Clone, Debug)]
pub struct NoopCache;

impl NoopCache {
    pub async fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CacheService for NoopCache {
    async fn find_user_agreement(&self, _user_id: i32, _group: &str) -> Result<Option<bool>> {
        Ok(None)
    }

    async fn store_user_agreement(&self, _user_id: i32, _group: &str, _agreed: bool) -> Result<()> {
        Ok(())
    }

    async fn get_latest_term_for_group(&self, _group: &str) -> Result<Option<TermOfUse>> {
        Ok(None)
    }

    async fn store_latest_term_for_group(&self, _term: &TermOfUse) -> Result<()> {
        Ok(())
    }

    async fn invalidate_cache_for_group(&self, _group: &str) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl HealthCheck for NoopCache {
    async fn ping(&self) -> Result<()> {
        Ok(())
    }
}

impl CacheServiceWithHealthCheck for NoopCache {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_ping_should_always_succeed() {
        let cache = NoopCache::new().await;

        let result = cache.ping().await;

        assert!(
            result.is_ok(),
            "NoopCache.ping() should always return Ok(())"
        );
    }

    #[tokio::test]
    async fn find_user_agreement_should_always_return_none() {
        let cache = NoopCache::new().await;

        let result = cache.find_user_agreement(1, "privacy-policy").await;

        assert!(result.is_ok(), "find_user_agreement should return Ok(None)");
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn store_user_agreement_should_always_succeed() {
        let cache = NoopCache::new().await;

        let result = cache.store_user_agreement(1, "privacy-policy", true).await;

        assert!(
            result.is_ok(),
            "store_user_agreement should always return Ok(())"
        );
    }

    #[tokio::test]
    async fn get_latest_term_for_group_should_always_return_none() {
        let cache = NoopCache::new().await;

        let result = cache.get_latest_term_for_group("privacy-policy").await;

        assert!(
            result.is_ok(),
            "get_latest_term_for_group should return Ok(None)"
        );
        assert!(
            result.unwrap().is_none(),
            "result should be None for noop cache"
        );
    }

    #[tokio::test]
    async fn invalidate_cache_for_group_should_always_succeed() {
        let cache = NoopCache::new().await;

        let result = cache.invalidate_cache_for_group("privacy-policy").await;

        assert!(
            result.is_ok(),
            "invalidate_cache_for_group should always return Ok(())"
        );
    }
}
