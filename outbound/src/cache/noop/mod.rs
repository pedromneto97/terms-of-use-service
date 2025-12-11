use domain::{
    data::service::CacheService,
    entities::TermOfUse,
    errors::{Result, TermsOfUseError},
};

#[derive(Clone, Debug)]
pub struct NoopCache;

impl NoopCache {
    pub async fn new() -> Self {
        Self
    }
}

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
