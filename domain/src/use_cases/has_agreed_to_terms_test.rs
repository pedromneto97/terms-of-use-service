#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::predicate::*;

    use crate::{
        data::{
            repository::{MockTermRepository, MockUserAgreementRepository},
            service::MockCacheService,
        },
        entities::TermOfUse,
        errors::{Result, TermsOfUseError},
        use_cases::has_user_agreed_to_term_use_case,
    };

    // Combined mock for testing
    struct MockCombinedRepository {
        term_repo: MockTermRepository,
        agreement_repo: MockUserAgreementRepository,
    }

    #[async_trait]
    impl crate::data::repository::TermRepository for MockCombinedRepository {
        async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<TermOfUse>> {
            self.term_repo.get_latest_term_for_group(group).await
        }

        async fn get_term_by_id(&self, term_id: i32) -> Result<Option<TermOfUse>> {
            self.term_repo.get_term_by_id(term_id).await
        }

        async fn create_term(&self, term: TermOfUse) -> Result<TermOfUse> {
            self.term_repo.create_term(term).await
        }
    }

    #[async_trait]
    impl crate::data::repository::UserAgreementRepository for MockCombinedRepository {
        async fn has_user_agreed_to_term(&self, user_id: i32, term_id: i32) -> Result<bool> {
            self.agreement_repo
                .has_user_agreed_to_term(user_id, term_id)
                .await
        }

        async fn create_user_agreement(&self, user_id: i32, term_id: i32) -> Result<()> {
            self.agreement_repo
                .create_user_agreement(user_id, term_id)
                .await
        }
    }

    impl crate::data::repository::DatabaseRepository for MockCombinedRepository {}

    #[tokio::test]
    async fn test_has_agreed_from_cache_true() {
        // Arrange
        let term_repo = MockTermRepository::new();
        let agreement_repo = MockUserAgreementRepository::new();
        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .with(eq(100), eq("privacy-policy"))
            .times(1)
            .returning(|_, _| Ok(Some(true)));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_has_agreed_from_cache_false() {
        // Arrange
        let term_repo = MockTermRepository::new();
        let agreement_repo = MockUserAgreementRepository::new();
        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(Some(false)));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_has_agreed_from_repository_true() {
        // Arrange
        let latest_term = TermOfUse {
            id: 15,
            group: "privacy-policy".to_string(),
            version: 4,
            url: "uploads/privacy-v4.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .with(eq("privacy-policy"))
            .times(1)
            .returning(move |_| Ok(Some(latest_term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_has_user_agreed_to_term()
            .with(eq(100), eq(15))
            .times(1)
            .returning(|_, _| Ok(true));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        cache
            .expect_store_user_agreement()
            .with(eq(100), eq("privacy-policy"), eq(true))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_has_agreed_from_repository_false() {
        // Arrange
        let latest_term = TermOfUse {
            id: 15,
            group: "privacy-policy".to_string(),
            version: 4,
            url: "uploads/privacy-v4.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(latest_term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_has_user_agreed_to_term()
            .returning(|_, _| Ok(false));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        cache
            .expect_store_user_agreement()
            .with(eq(100), eq("privacy-policy"), eq(false))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_has_agreed_term_not_found() {
        // Arrange
        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        let agreement_repo = MockUserAgreementRepository::new();
        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        let user_id = 100;
        let group = "non-existent-group";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::NotFound);
    }

    #[tokio::test]
    async fn test_has_agreed_get_latest_failure() {
        // Arrange
        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        let agreement_repo = MockUserAgreementRepository::new();
        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::InternalServerError);
    }

    #[tokio::test]
    async fn test_has_agreed_repository_check_failure() {
        // Arrange
        let latest_term = TermOfUse {
            id: 15,
            group: "privacy-policy".to_string(),
            version: 4,
            url: "uploads/privacy-v4.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(latest_term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_has_user_agreed_to_term()
            .returning(|_, _| Err(TermsOfUseError::InternalServerError));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::InternalServerError);
    }

    #[tokio::test]
    async fn test_has_agreed_cache_find_failure_fallback_to_repository() {
        // Arrange
        let latest_term = TermOfUse {
            id: 15,
            group: "privacy-policy".to_string(),
            version: 4,
            url: "uploads/privacy-v4.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(latest_term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_has_user_agreed_to_term()
            .returning(|_, _| Ok(true));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Err(TermsOfUseError::InternalServerError));

        cache
            .expect_store_user_agreement()
            .returning(|_, _, _| Ok(()));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert - Should succeed by falling back to repository
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_has_agreed_cache_store_failure_doesnt_affect_result() {
        // Arrange
        let latest_term = TermOfUse {
            id: 15,
            group: "privacy-policy".to_string(),
            version: 4,
            url: "uploads/privacy-v4.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(latest_term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_has_user_agreed_to_term()
            .returning(|_, _| Ok(true));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        cache
            .expect_store_user_agreement()
            .returning(|_, _, _| Err(TermsOfUseError::InternalServerError));

        let user_id = 100;
        let group = "privacy-policy";

        // Act
        let result = has_user_agreed_to_term_use_case(&repository, &cache, user_id, group).await;

        // Assert - Should succeed despite cache store failure
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_has_agreed_different_users_and_groups() {
        // Arrange
        let latest_term = TermOfUse {
            id: 15,
            group: "group-a".to_string(),
            version: 4,
            url: "uploads/privacy-v4.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut term_repo = MockTermRepository::new();
        term_repo
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(latest_term.clone())));

        let mut agreement_repo = MockUserAgreementRepository::new();
        agreement_repo
            .expect_has_user_agreed_to_term()
            .returning(|_, _| Ok(true));

        let repository = MockCombinedRepository {
            term_repo,
            agreement_repo,
        };

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .returning(|_, _| Ok(None));

        cache
            .expect_store_user_agreement()
            .returning(|_, _, _| Ok(()));

        // Act & Assert - User 1, Group A
        let result1 = has_user_agreed_to_term_use_case(&repository, &cache, 1, "group-a").await;
        assert!(result1.is_ok());

        // Act & Assert - User 2, Group A
        let result2 = has_user_agreed_to_term_use_case(&repository, &cache, 2, "group-a").await;
        assert!(result2.is_ok());

        // Act & Assert - User 1, Group B
        let result3 = has_user_agreed_to_term_use_case(&repository, &cache, 1, "group-b").await;
        assert!(result3.is_ok());
    }
}
