use crate::domain::{entities::TermOfUse, errors::TermsOfUseError};

pub trait TermRepository {
    async fn get_latest_term_for_group(
        &self,
        group: &str,
    ) -> Result<Option<TermOfUse>, TermsOfUseError>;

    async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>, TermsOfUseError>;

    async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse, TermsOfUseError>;
}

pub trait UserAgreementRepository {
    async fn has_user_agreed_to_term(
        &self,
        user_id: i32,
        term_id: i32,
    ) -> Result<bool, TermsOfUseError>;

    async fn create_user_agreement(
        &self,
        user_id: i32,
        term_id: i32,
    ) -> Result<(), TermsOfUseError>;
}
