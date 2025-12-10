use aws_sdk_dynamodb::types::AttributeValue;
use tracing::error;

use crate::{
    domain::{data::repository::TermRepository, entities::TermOfUse, errors::TermsOfUseError},
    outbound::database::dynamodb::{
        DynamoDBClient, migration::GSI_TERMS_GROUP_VERSION, model::TERMS_TABLE,
    },
};

impl TermRepository for DynamoDBClient {
    #[tracing::instrument(skip(self, group))]
    async fn get_latest_term_for_group(
        &self,
        group: &str,
    ) -> Result<Option<TermOfUse>, TermsOfUseError> {
        let value = self
            .client
            .query()
            .table_name(TERMS_TABLE)
            .index_name(GSI_TERMS_GROUP_VERSION)
            .key_condition_expression("#group = :group")
            .expression_attribute_names("#group", "group")
            .expression_attribute_values(":group", AttributeValue::S(group.to_string()))
            .scan_index_forward(false) // Descending order to get the latest
            .limit(1)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to query latest term for group '{group}': {err}");

                TermsOfUseError::InternalServerError
            })?;

        if let Some(items) = value.items {
            if let Some(item) = items.first() {
                let term = TermOfUse::try_from(item)?;

                return Ok(Some(term));
            }
        }

        Ok(None)
    }

    #[tracing::instrument(skip(self, term_id))]
    async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>, TermsOfUseError> {
        let value = self
            .client
            .get_item()
            .table_name(TERMS_TABLE)
            .key("id", AttributeValue::N(term_id.to_string()))
            .send()
            .await
            .map_err(|err| {
                error!("Failed to get term by id '{term_id}': {err}");

                TermsOfUseError::InternalServerError
            })?;

        if let Some(item) = value.item {
            let term = TermOfUse::try_from(&item)?;

            return Ok(Some(term));
        }

        Ok(None)
    }

    #[tracing::instrument(skip(self, term))]
    async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse, TermsOfUseError> {
        // Generate next ID atomically
        let id = self.get_next_id("terms_id_counter").await?;

        let mut item = std::collections::HashMap::new();

        item.insert("id".to_string(), AttributeValue::N(id.to_string()));
        item.insert("group".to_string(), AttributeValue::S(term.group.clone()));
        item.insert("url".to_string(), AttributeValue::S(term.url.clone()));
        item.insert(
            "version".to_string(),
            AttributeValue::N(term.version.to_string()),
        );
        if let Some(info) = &term.info {
            item.insert("info".to_string(), AttributeValue::S(info.clone()));
        }
        item.insert(
            "created_at".to_string(),
            AttributeValue::N(term.created_at.and_utc().timestamp().to_string()),
        );

        self.client
            .put_item()
            .table_name(TERMS_TABLE)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|err| {
                error!("Failed to create term '{:?}': {err}", term);

                TermsOfUseError::InternalServerError
            })?;

        Ok(TermOfUse {
            id,
            group: term.group,
            url: term.url,
            version: term.version,
            info: term.info,
            created_at: term.created_at,
        })
    }
}
