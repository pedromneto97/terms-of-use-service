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
