use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    domain::{data::repository::UserAgreementRepository, errors::TermsOfUseError},
    outbound::database::postgres::{
        PostgresRepository,
        data::models::{prelude::UserAgreements, user_agreements},
    },
};

impl UserAgreementRepository for PostgresRepository {
    async fn has_user_agreed_to_term(
        &self,
        user_id: i32,
        term_id: i32,
    ) -> Result<bool, TermsOfUseError> {
        UserAgreements::find()
            .filter(user_agreements::Column::UserId.eq(user_id))
            .filter(user_agreements::Column::TermOfUseId.eq(term_id))
            .one(&self.db)
            .await
            .map(|agreement| agreement.is_some())
            .map_err(|_| TermsOfUseError::InternalServerError)
    }

    async fn create_user_agreement(
        &self,
        user_id: i32,
        term_id: i32,
    ) -> Result<(), TermsOfUseError> {
        let new_agreement = user_agreements::ActiveModel {
            user_id: sea_orm::Set(user_id),
            term_of_use_id: sea_orm::Set(term_id),
            agreed_at: sea_orm::Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        new_agreement
            .insert(&self.db)
            .await
            .map_err(|_| TermsOfUseError::InternalServerError)?;

        Ok(())
    }
}
