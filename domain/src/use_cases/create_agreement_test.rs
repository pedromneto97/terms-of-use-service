#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::predicate::*;

    use crate::{
        data::{
            repository::{MockTermRepository, MockUserAgreementRepository},
            service::{MockCacheService, MockPublisherService},
        },
        dto::AcceptedTermOfUseDTO,
        entities::TermOfUse,
        errors::TermsOfUseError,
        use_cases::create_user_agreement_use_case,
    };

    // Combined mock for testing
    struct MockCombinedRepository {
        term_repo: MockTermRepository,
        agreement_repo: MockUserAgreementRepository,
    }

    #[async_trait]
    impl crate::data::repository::TermRepository for MockCombinedRepository {
        async fn get_latest_term_for_group(
            &self,
            group: &str,
        ) -> Result<Option<TermOfUse>, TermsOfUseError> {
            self.term_repo.get_latest_term_for_group(group).await
        }

        async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>, TermsOfUseError> {
            self.term_repo.get_term_by_id(term_id).await
        }

        async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse, TermsOfUseError> {
            self.term_repo.create_term(term).await
        }
    }

    #[async_trait]
    impl crate::data::repository::UserAgreementRepository for MockCombinedRepository {
        async fn has_user_agreed_to_term(
            &self,
            user_id: i32,
            term_id: i32,
        ) -> Result<bool, TermsOfUseError> {
            self.agreement_repo
                .has_user_agreed_to_term(user_id, term_id)
                .await
        }

        async fn create_user_agreement(
            &self,
            user_id: i32,
            term_id: i32,
        ) -> Result<(), TermsOfUseError> {
            self.agreement_repo
                .create_user_agreement(user_id, term_id)
                .await
        }
    }

    impl crate::data::repository::DatabaseRepository for MockCombinedRepository {}

    #[tokio::test]
    async fn test_create_user_agreement_success() {
        // Arrange
        let term = TermOfUse {
            id: 10,
            group: "privacy-policy".to_string(),
            version: 2,
            url: "uploads/privacy-v2.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_term_by_id()
            .with(eq(10))
            .times(1)
            .returning(move |_| Ok(Some(term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_create_user_agreement()
            .with(eq(42), eq(10))
            .times(1)
            .returning(|_, _| Ok(()));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_store_user_agreement()
            .with(eq(42), eq("privacy-policy"), eq(true))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher
            .expect_publish_agreement()
            .times(1)
            .withf(|dto: &AcceptedTermOfUseDTO| {
                dto.user_id == 42 && dto.term_id == 10 && dto.group == "privacy-policy"
            })
            .returning(|_| Ok(()));

        let user_id = 42;
        let term_id = 10;

        // Act
        let result =
            create_user_agreement_use_case(&repository, &cache, &publisher, user_id, term_id).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_user_agreement_term_not_found() {
        // Arrange
        let mut term_repo = MockTermRepository::new();
        term_repo.expect_get_term_by_id().returning(|_| Ok(None));

        let agreement_repo = MockUserAgreementRepository::new();
        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let cache = MockCacheService::new();
        let publisher = MockPublisherService::new();

        let user_id = 42;
        let term_id = 999;

        // Act
        let result =
            create_user_agreement_use_case(&repository, &cache, &publisher, user_id, term_id).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::NotFound);
    }

    #[tokio::test]
    async fn test_create_user_agreement_get_term_failure() {
        // Arrange
        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_term_by_id()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        let agreement_repo = MockUserAgreementRepository::new();
        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let cache = MockCacheService::new();
        let publisher = MockPublisherService::new();

        let user_id = 42;
        let term_id = 10;

        // Act
        let result =
            create_user_agreement_use_case(&repository, &cache, &publisher, user_id, term_id).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::InternalServerError);
    }

    #[tokio::test]
    async fn test_create_user_agreement_create_agreement_failure() {
        // Arrange
        let term = TermOfUse {
            id: 10,
            group: "privacy-policy".to_string(),
            version: 2,
            url: "uploads/privacy-v2.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_term_by_id()
            .returning(move |_| Ok(Some(term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_create_user_agreement()
            .returning(|_, _| Err(TermsOfUseError::InternalServerError));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let cache = MockCacheService::new();
        let publisher = MockPublisherService::new();

        let user_id = 42;
        let term_id = 10;

        // Act
        let result =
            create_user_agreement_use_case(&repository, &cache, &publisher, user_id, term_id).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::InternalServerError);
    }

    #[tokio::test]
    async fn test_create_user_agreement_cache_failure_doesnt_affect_result() {
        // Arrange
        let term = TermOfUse {
            id: 10,
            group: "privacy-policy".to_string(),
            version: 2,
            url: "uploads/privacy-v2.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_term_by_id()
            .returning(move |_| Ok(Some(term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_create_user_agreement()
            .returning(|_, _| Ok(()));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_store_user_agreement()
            .returning(|_, _, _| Err(TermsOfUseError::InternalServerError));

        let mut publisher = MockPublisherService::new();
        publisher.expect_publish_agreement().returning(|_| Ok(()));

        let user_id = 42;
        let term_id = 10;

        // Act
        let result =
            create_user_agreement_use_case(&repository, &cache, &publisher, user_id, term_id).await;

        // Assert - Should succeed despite cache failure
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_user_agreement_publisher_failure_doesnt_affect_result() {
        // Arrange
        let term = TermOfUse {
            id: 10,
            group: "privacy-policy".to_string(),
            version: 2,
            url: "uploads/privacy-v2.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_term_by_id()
            .returning(move |_| Ok(Some(term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_create_user_agreement()
            .returning(|_, _| Ok(()));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_store_user_agreement()
            .returning(|_, _, _| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher
            .expect_publish_agreement()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        let user_id = 42;
        let term_id = 10;

        // Act
        let result =
            create_user_agreement_use_case(&repository, &cache, &publisher, user_id, term_id).await;

        // Assert - Should succeed despite publisher failure
        assert!(result.is_ok());
    }
}
