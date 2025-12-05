use crate::domain::{entities::TermOfUse, errors::TermsOfUseError};

pub trait CacheService: Send + Sync {
    async fn find_user_agreement(
        &self,
        user_id: i32,
        group: &str,
    ) -> Result<Option<bool>, TermsOfUseError>;

    async fn store_user_agreement(
        &self,
        user_id: i32,
        group: &str,
        agreed: bool,
    ) -> Result<(), TermsOfUseError>;

    async fn get_latest_term_for_group(
        &self,
        group: &str,
    ) -> Result<Option<TermOfUse>, TermsOfUseError>;

    async fn store_latest_term_for_group(&self, term: &TermOfUse) -> Result<(), TermsOfUseError>;

    async fn invalidate_cache_for_group(&self, group: &str) -> Result<(), TermsOfUseError>;
}

#[derive(Clone, Debug)]
pub struct NoopCacheService;

impl NoopCacheService {
    pub async fn new() -> Self {
        NoopCacheService
    }
}

impl CacheService for NoopCacheService {
    async fn find_user_agreement(
        &self,
        _user_id: i32,
        _group: &str,
    ) -> Result<Option<bool>, TermsOfUseError> {
        Ok(None)
    }

    async fn store_user_agreement(
        &self,
        _user_id: i32,
        _group: &str,
        _agreed: bool,
    ) -> Result<(), TermsOfUseError> {
        Ok(())
    }

    async fn get_latest_term_for_group(
        &self,
        _group: &str,
    ) -> Result<Option<TermOfUse>, TermsOfUseError> {
        Ok(None)
    }

    async fn store_latest_term_for_group(&self, _term: &TermOfUse) -> Result<(), TermsOfUseError> {
        Ok(())
    }

    async fn invalidate_cache_for_group(&self, _group: &str) -> Result<(), TermsOfUseError> {
        Ok(())
    }
}
