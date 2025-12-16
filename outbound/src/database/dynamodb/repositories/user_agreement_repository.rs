use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use domain::{
    data::repository::UserAgreementRepository,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use crate::database::dynamodb::{DynamoRepository, model::USER_AGREEMENTS_TABLE};

#[async_trait]
impl UserAgreementRepository for DynamoRepository {
    #[tracing::instrument(skip(self, user_id, term_id))]
    async fn has_user_agreed_to_term(&self, user_id: i32, term_id: i32) -> Result<bool> {
        let agreement_key = format!("{user_id}#{term_id}");

        let result = self
            .client
            .get_item()
            .table_name(USER_AGREEMENTS_TABLE)
            .key("agreement_key", AttributeValue::S(agreement_key.clone()))
            .send()
            .await
            .map_err(|err| {
                error!("Failed to check user agreement for key '{agreement_key}': {err}");

                TermsOfUseError::InternalServerError
            })?;

        Ok(result.item.is_some())
    }

    #[tracing::instrument(skip(self, user_id, term_id))]
    async fn create_user_agreement(&self, user_id: i32, term_id: i32) -> Result<()> {
        let agreement_key = format!("{user_id}#{term_id}");

        let mut item = std::collections::HashMap::new();

        item.insert(
            "agreement_key".to_string(),
            AttributeValue::S(agreement_key.clone()),
        );
        item.insert(
            "user_id".to_string(),
            AttributeValue::N(user_id.to_string()),
        );
        item.insert(
            "term_id".to_string(),
            AttributeValue::N(term_id.to_string()),
        );
        item.insert(
            "agreed_at".to_string(),
            AttributeValue::S(Utc::now().naive_utc().to_string()),
        );

        self.client
            .put_item()
            .table_name(USER_AGREEMENTS_TABLE)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|err| {
                error!("Failed to create user agreement for key '{agreement_key}': {err}");

                TermsOfUseError::InternalServerError
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use domain::data::repository::UserAgreementRepository;

    use crate::database::dynamodb::DynamoRepository;

    async fn create_test_repository() -> DynamoRepository {
        DynamoRepository::new().await
    }

    #[tokio::test]
    async fn test_has_user_agreed_to_term_returns_false_when_no_agreement() {
        let repo = create_test_repository().await;

        let result = repo.has_user_agreed_to_term(1, 1).await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_create_user_agreement_succeeds() {
        let repo = create_test_repository().await;

        let result = repo.create_user_agreement(123, 456).await;

        assert!(result.is_ok());

        let check_result = repo.has_user_agreed_to_term(123, 456).await;
        assert!(check_result.is_ok());
        assert!(check_result.unwrap());
    }
}
