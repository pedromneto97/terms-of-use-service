#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mockall::predicate::*;

    use crate::{
        data::{
            repository::MockTermRepository,
            service::{MockCacheService, MockStorageService},
        },
        entities::TermOfUse,
        errors::TermsOfUseError,
        use_cases::get_latest_term_use_case,
    };

    #[tokio::test]
    async fn test_get_latest_term_from_cache() {
        // Arrange
        let cached_term = TermOfUse {
            id: 10,
            group: "privacy-policy".to_string(),
            version: 5,
            url: "https://storage.example.com/cached.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: Some("Cached version".to_string()),
        };

        let repository = MockTermRepository::new();

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .with(eq("privacy-policy"))
            .times(1)
            .returning(move |_| Ok(Some(cached_term.clone())));

        let storage = MockStorageService::new();

        // Act
        let result =
            get_latest_term_use_case(&repository, &cache, &storage, "privacy-policy").await;

        // Assert
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.id, 10);
        assert_eq!(term.version, 5);
        assert_eq!(term.url, "https://storage.example.com/cached.pdf");
    }

    #[tokio::test]
    async fn test_get_latest_term_from_repository() {
        // Arrange
        let db_term = TermOfUse {
            id: 5,
            group: "privacy-policy".to_string(),
            version: 3,
            url: "uploads/privacy-v3.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: Some("Latest version".to_string()),
        };

        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .with(eq("privacy-policy"))
            .times(1)
            .returning(move |_| Ok(Some(db_term.clone())));

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        cache
            .expect_store_latest_term_for_group()
            .times(1)
            .returning(|_| Ok(()));

        let mut storage = MockStorageService::new();
        storage
            .expect_get_file_url()
            .with(eq("uploads/privacy-v3.pdf"))
            .times(1)
            .returning(|_| Ok("https://storage.example.com/privacy-v3.pdf".to_string()));

        // Act
        let result =
            get_latest_term_use_case(&repository, &cache, &storage, "privacy-policy").await;

        // Assert
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.id, 5);
        assert_eq!(term.version, 3);
        assert_eq!(term.url, "https://storage.example.com/privacy-v3.pdf");
        assert_eq!(term.info, Some("Latest version".to_string()));
    }

    #[tokio::test]
    async fn test_get_latest_term_not_found() {
        // Arrange
        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        let storage = MockStorageService::new();

        // Act
        let result =
            get_latest_term_use_case(&repository, &cache, &storage, "non-existent-group").await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::NotFound);
    }

    #[tokio::test]
    async fn test_get_latest_term_repository_failure() {
        // Arrange
        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        let storage = MockStorageService::new();

        // Act
        let result =
            get_latest_term_use_case(&repository, &cache, &storage, "privacy-policy").await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::InternalServerError);
    }

    #[tokio::test]
    async fn test_get_latest_term_cache_failure_fallback_to_repository() {
        // Arrange
        let db_term = TermOfUse {
            id: 5,
            group: "privacy-policy".to_string(),
            version: 3,
            url: "uploads/privacy-v3.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(db_term.clone())));

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        cache
            .expect_store_latest_term_for_group()
            .returning(|_| Ok(()));

        let mut storage = MockStorageService::new();
        storage
            .expect_get_file_url()
            .returning(|_| Ok("https://storage.example.com/privacy-v3.pdf".to_string()));

        // Act
        let result =
            get_latest_term_use_case(&repository, &cache, &storage, "privacy-policy").await;

        // Assert - Should succeed by falling back to repository
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.id, 5);
    }

    #[tokio::test]
    async fn test_get_latest_term_storage_cache_failure_doesnt_affect_result() {
        // Arrange
        let db_term = TermOfUse {
            id: 5,
            group: "privacy-policy".to_string(),
            version: 3,
            url: "uploads/privacy-v3.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(db_term.clone())));

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        cache
            .expect_store_latest_term_for_group()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        let mut storage = MockStorageService::new();
        storage
            .expect_get_file_url()
            .returning(|_| Ok("https://storage.example.com/privacy-v3.pdf".to_string()));

        // Act
        let result =
            get_latest_term_use_case(&repository, &cache, &storage, "privacy-policy").await;

        // Assert - Should succeed despite cache store failure
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.id, 5);
    }
}
