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
