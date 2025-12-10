use crate::{entities::TermOfUse, errors::Result};

#[cfg_attr(test, mockall::automock)]
pub trait TermRepository: Send + Sync {
    async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<TermOfUse>>;

    async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>>;

    async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse>;
}

#[cfg_attr(test, mockall::automock)]
pub trait UserAgreementRepository: Send + Sync {
    async fn has_user_agreed_to_term(&self, user_id: i32, term_id: i32) -> Result<bool>;

    async fn create_user_agreement(&self, user_id: i32, term_id: i32) -> Result<()>;
}
