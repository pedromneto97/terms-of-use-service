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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::errors::TermsOfUseError;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    use super::*;

    #[tokio::test]
    async fn has_user_agreed_to_term_returns_true_when_found() {
        let agreement = user_agreements::Model {
            id: 1,
            term_of_use_id: 2,
            user_id: 3,
            agreed_at: Utc::now().naive_utc(),
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![agreement]])
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.has_user_agreed_to_term(3, 2).await.unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn has_user_agreed_to_term_returns_false_when_missing() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![Vec::<user_agreements::Model>::new()])
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.has_user_agreed_to_term(3, 2).await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn has_user_agreed_to_term_propagates_error() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(Vec::<Vec<user_agreements::Model>>::new())
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.has_user_agreed_to_term(3, 2).await;

        assert!(matches!(result, Err(TermsOfUseError::InternalServerError)));
    }

    #[tokio::test]
    async fn create_user_agreement_inserts_record() {
        let inserted = user_agreements::Model {
            id: 11,
            term_of_use_id: 5,
            user_id: 9,
            agreed_at: Utc::now().naive_utc(),
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![inserted]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 11,
                rows_affected: 1,
            }])
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.create_user_agreement(9, 5).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_user_agreement_propagates_error() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(Vec::<Vec<user_agreements::Model>>::new())
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.create_user_agreement(9, 5).await;

        assert!(matches!(result, Err(TermsOfUseError::InternalServerError)));
    }
}
