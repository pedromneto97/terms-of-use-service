use async_trait::async_trait;
use chrono::Utc;
use domain::{
    data::repository::UserAgreementRepository,
    errors::{Result, TermsOfUseError},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use tracing::error;

use crate::database::postgres::{
    PostgresRepository,
    data::models::{prelude::UserAgreements, user_agreements},
};

#[async_trait]
impl UserAgreementRepository for PostgresRepository {
    #[tracing::instrument(skip(self, user_id, term_id))]
    async fn has_user_agreed_to_term(&self, user_id: i32, term_id: i32) -> Result<bool> {
        UserAgreements::find()
            .filter(user_agreements::Column::UserId.eq(user_id))
            .filter(user_agreements::Column::TermOfUseId.eq(term_id))
            .one(&self.db)
            .await
            .map(|agreement| agreement.is_some())
            .map_err(|err| {
                error!("Failed to check user agreement: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    #[tracing::instrument(skip(self, user_id, term_id))]
    async fn create_user_agreement(&self, user_id: i32, term_id: i32) -> Result<()> {
        let new_agreement = user_agreements::ActiveModel {
            user_id: sea_orm::Set(user_id),
            term_of_use_id: sea_orm::Set(term_id),
            agreed_at: sea_orm::Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        new_agreement.insert(&self.db).await.map_err(|err| {
            error!("Failed to create user agreement: {err}");

            TermsOfUseError::InternalServerError
        })?;

        Ok(())
    }
}
