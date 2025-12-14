use aws_sdk_dynamodb::types::AttributeValue;
use domain::{data::repository::TermRepository, entities::TermOfUse, errors::TermsOfUseError};
use tracing::error;

use crate::database::dynamodb::{
    DynamoRepository,
    migration::GSI_TERMS_GROUP_VERSION,
    model::{TERMS_TABLE, map_term_from_item},
};

impl TermRepository for DynamoRepository {
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

        if let Some(items) = value.items
            && let Some(item) = items.first()
        {
            let term = map_term_from_item(item)?;

            return Ok(Some(term));
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
            let term = map_term_from_item(&item)?;

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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::{data::repository::TermRepository, entities::TermOfUse};

    use crate::database::dynamodb::DynamoRepository;

    async fn create_test_repository() -> DynamoRepository {
        DynamoRepository::new().await
    }

    fn create_sample_term(id: i32, group: &str, version: u32) -> TermOfUse {
        TermOfUse {
            id,
            group: group.to_string(),
            url: format!("https://example.com/terms/{group}/v{version}"),
            version,
            info: Some(format!("Test term for {group} v{version}")),
            created_at: Utc::now().naive_utc(),
        }
    }

    #[tokio::test]
    async fn test_should_return_none_when_no_terms_exist() {
        let repo = create_test_repository().await;

        let result = repo.get_latest_term_for_group("nonexistent-group").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_should_return_none_when_term_id_not_found() {
        let repo = create_test_repository().await;

        let result = repo.get_term_by_id(99999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_create_term_successful() {
        let repo = create_test_repository().await;

        let created_at = Utc::now().naive_utc();
        let term = TermOfUse {
            id: 0, // ID should be auto-generated
            group: "test-group".to_string(),
            url: "https://example.com/terms/v1".to_string(),
            version: 1,
            info: Some("Test term".to_string()),
            created_at: created_at,
        };

        let result = repo.create_term(term).await.unwrap();

        assert_eq!(result.group, "test-group");
        assert_eq!(result.version, 1);
        assert_eq!(result.info.unwrap(), "Test term");
        assert_eq!(result.created_at, created_at);
        assert!(result.id > 0);
    }

    #[tokio::test]
    async fn test_get_term_by_id_retrieves_existing_term() {
        let repo = create_test_repository().await;

        let term = create_sample_term(0, "terms-of-service", 1);
        let created_term = repo.create_term(term).await.unwrap();

        let retrieved_term = repo
            .get_term_by_id(created_term.id)
            .await
            .unwrap()
            .expect("Term should exist");

        assert_eq!(retrieved_term.id, created_term.id);
        assert_eq!(retrieved_term.group, created_term.group);
        assert_eq!(retrieved_term.version, created_term.version);
        assert_eq!(retrieved_term.url, created_term.url);
        assert_eq!(retrieved_term.info, created_term.info);
        assert_eq!(
            retrieved_term.created_at.and_utc().timestamp(),
            created_term.created_at.and_utc().timestamp()
        );
    }

    #[tokio::test]
    async fn test_get_latest_term_for_group_returns_highest_version() {
        let repo = create_test_repository().await;

        let group = "cookie-policy";

        // Create multiple versions
        let term_v1 = create_sample_term(0, group, 1);
        let term_v2 = create_sample_term(0, group, 2);
        let term_v3 = create_sample_term(0, group, 3);

        repo.create_term(term_v1).await.expect("v1 created");
        repo.create_term(term_v2).await.expect("v2 created");
        repo.create_term(term_v3).await.expect("v3 created");

        let result = repo
            .get_latest_term_for_group(group)
            .await
            .unwrap()
            .expect("Latest term should exist");

        assert_eq!(result.group, group);
        assert_eq!(result.version, 3);
    }
}
