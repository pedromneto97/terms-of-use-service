use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{
    domain::{data::repository::TermRepository, entities::TermOfUse, errors::TermsOfUseError},
    outbound::database::postgres::{
        PostgresRepository,
        data::models::{prelude::Terms, terms},
    },
};

impl TermRepository for PostgresRepository {
    async fn get_latest_term_for_group(
        &self,
        group: &str,
    ) -> Result<Option<TermOfUse>, TermsOfUseError> {
        Terms::find()
            .filter(terms::Column::Group.eq(group))
            .order_by_desc(terms::Column::Version)
            .one(&self.db)
            .await
            .map(|term| term.map(Into::into))
            .map_err(|_| TermsOfUseError::InternalServerError)
    }

    async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>, TermsOfUseError> {
        Terms::find_by_id(term_id)
            .one(&self.db)
            .await
            .map(|term| term.map(Into::into))
            .map_err(|_| TermsOfUseError::InternalServerError)
    }

    async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse, TermsOfUseError> {
        let new_term = terms::ActiveModel {
            url: sea_orm::Set(term.url),
            group: sea_orm::Set(term.group),
            info: sea_orm::Set(term.info),
            version: sea_orm::Set(term.version as i32),
            created_at: sea_orm::Set(term.created_at),
            ..Default::default()
        };

        let inserted_term = new_term
            .insert(&self.db)
            .await
            .map_err(|_| TermsOfUseError::InternalServerError)?;

        Ok(inserted_term.into())
    }
}
