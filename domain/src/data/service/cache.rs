use crate::{entities::TermOfUse, errors::Result};

#[cfg_attr(test, mockall::automock)]
pub trait CacheService: Send + Sync {
    async fn find_user_agreement(&self, user_id: i32, group: &str) -> Result<Option<bool>>;

    async fn store_user_agreement(&self, user_id: i32, group: &str, agreed: bool) -> Result<()>;

    async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<TermOfUse>>;

    async fn store_latest_term_for_group(&self, term: &TermOfUse) -> Result<()>;

    async fn invalidate_cache_for_group(&self, group: &str) -> Result<()>;
}
