use async_trait::async_trait;
use domain::{
    data::repository::TermRepository,
    entities::TermOfUse,
    errors::{Result, TermsOfUseError},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use tracing::error;

use crate::database::postgres::{
    PostgresRepository,
    data::models::{prelude::Terms, terms},
};

#[async_trait]
impl TermRepository for PostgresRepository {
    #[tracing::instrument(skip(self, group))]
    async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<TermOfUse>> {
        Terms::find()
            .filter(terms::Column::Group.eq(group))
            .order_by_desc(terms::Column::Version)
            .one(&self.db)
            .await
            .map(|term| term.map(Into::into))
            .map_err(|err| {
                error!("Failed to fetch latest term for group {group}: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    #[tracing::instrument(skip(self, term_id))]
    async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>> {
        Terms::find_by_id(term_id)
            .one(&self.db)
            .await
            .map(|term| term.map(Into::into))
            .map_err(|err| {
                error!("Failed to fetch term by id {term_id}: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    #[tracing::instrument(skip(self, term))]
    async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse> {
        let new_term = terms::ActiveModel {
            url: sea_orm::Set(term.url),
            group: sea_orm::Set(term.group),
            info: sea_orm::Set(term.info),
            version: sea_orm::Set(term.version as i32),
            created_at: sea_orm::Set(term.created_at),
            ..Default::default()
        };

        let inserted_term = new_term.insert(&self.db).await.map_err(|err| {
            error!("Failed to create new term: {err}");

            TermsOfUseError::InternalServerError
        })?;

        Ok(inserted_term.into())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::errors::TermsOfUseError;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn get_latest_term_for_group_maps_row() {
        let created_at = Utc::now().naive_utc();

        let term_model = terms::Model {
            id: 2,
            url: "https://example.com/terms-v2".to_string(),
            group: "consumer".to_string(),
            version: 2,
            info: Some("v2 info".to_string()),
            created_at,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![term_model.clone()]])
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository
            .get_latest_term_for_group("consumer")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.id, term_model.id);
        assert_eq!(result.url, term_model.url);
        assert_eq!(result.group, term_model.group);
        assert_eq!(result.version, term_model.version as u32);
        assert_eq!(result.info, term_model.info);
        assert_eq!(result.created_at, term_model.created_at);
    }

    #[tokio::test]
    #[test_log::test]
    async fn get_latest_term_for_group_propagates_error() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(Vec::<Vec<terms::Model>>::new())
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.get_latest_term_for_group("missing").await;

        assert!(matches!(result, Err(TermsOfUseError::InternalServerError)));
    }

    #[tokio::test]
    #[test_log::test]
    async fn get_term_by_id_returns_none_for_missing() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![Vec::<terms::Model>::new()])
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.get_term_by_id(42).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    #[test_log::test]
    async fn create_term_returns_inserted_term() {
        let created_at = Utc::now().naive_utc();
        let input = TermOfUse {
            id: 0,
            url: "https://example.com/terms".to_string(),
            group: "merchant".to_string(),
            version: 1,
            info: None,
            created_at,
        };

        let inserted = terms::Model {
            id: 10,
            url: input.url.clone(),
            group: input.group.clone(),
            version: input.version as i32,
            info: input.info.clone(),
            created_at: input.created_at,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![inserted.clone()]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: inserted.id as u64,
                rows_affected: 1,
            }])
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.create_term(input).await.unwrap();

        assert_eq!(result.id, inserted.id);
        assert_eq!(result.group, inserted.group);
        assert_eq!(result.url, inserted.url);
        assert_eq!(result.version, inserted.version as u32);
        assert_eq!(result.info, inserted.info);
        assert_eq!(result.created_at, inserted.created_at);
    }

    #[tokio::test]
    #[test_log::test]
    async fn create_term_propagates_error() {
        let created_at = Utc::now().naive_utc();
        let input = TermOfUse {
            id: 0,
            url: "https://example.com/terms".to_string(),
            group: "merchant".to_string(),
            version: 1,
            info: None,
            created_at,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(Vec::<Vec<terms::Model>>::new())
            .into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.create_term(input).await;

        assert!(matches!(result, Err(TermsOfUseError::InternalServerError)));
    }
}
