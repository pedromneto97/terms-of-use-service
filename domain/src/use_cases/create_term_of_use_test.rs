#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mockall::predicate::*;
    use std::path::Path;

    use crate::{
        data::{
            repository::MockTermRepository,
            service::{MockCacheService, MockStorageService},
        },
        dto::CreateTermOfUseDTO,
        entities::TermOfUse,
        errors::TermsOfUseError,
        use_cases::create_term_of_use_use_case,
    };

    #[tokio::test]
    async fn test_create_first_term_of_use_success() {
        // Arrange
        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .with(eq("privacy-policy"))
            .times(1)
            .returning(|_| Ok(None));

        repository
            .expect_create_term()
            .times(1)
            .returning(|mut term| {
                term.id = 123;
                Ok(term)
            });

        let mut storage = MockStorageService::new();
        storage
            .expect_upload_file()
            .times(1)
            .returning(|_, _| Ok("uploads/test-file.pdf".to_string()));

        storage
            .expect_get_file_url()
            .with(eq("uploads/test-file.pdf"))
            .times(1)
            .returning(|_| Ok("https://storage.example.com/test-file.pdf".to_string()));

        let mut cache = MockCacheService::new();
        cache
            .expect_invalidate_cache_for_group()
            .with(eq("privacy-policy"))
            .times(1)
            .returning(|_| Ok(()));

        let dto = CreateTermOfUseDTO {
            group: "privacy-policy".to_string(),
            info: Some("Initial version".to_string()),
        };

        let file_path = Path::new("/tmp/test.pdf");

        // Act
        let result = create_term_of_use_use_case(
            &repository,
            &storage,
            &cache,
            dto,
            file_path,
            "application/pdf",
        )
        .await;

        // Assert
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.id, 123);
        assert_eq!(term.group, "privacy-policy");
        assert_eq!(term.version, 1); // First version
        assert_eq!(term.url, "https://storage.example.com/test-file.pdf");
        assert_eq!(term.info, Some("Initial version".to_string()));
    }

    #[tokio::test]
    async fn test_create_term_of_use_increments_version() {
        // Arrange
        let existing_term = TermOfUse {
            id: 1,
            group: "privacy-policy".to_string(),
            version: 3,
            url: "uploads/old-file.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        };

        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(move |_| Ok(Some(existing_term.clone())));

        repository.expect_create_term().returning(|mut term| {
            term.id = 2;
            Ok(term)
        });

        let mut storage = MockStorageService::new();
        storage
            .expect_upload_file()
            .returning(|_, _| Ok("uploads/test-file.pdf".to_string()));

        storage
            .expect_get_file_url()
            .returning(|_| Ok("https://storage.example.com/test-file.pdf".to_string()));

        let mut cache = MockCacheService::new();
        cache
            .expect_invalidate_cache_for_group()
            .returning(|_| Ok(()));

        let dto = CreateTermOfUseDTO {
            group: "privacy-policy".to_string(),
            info: Some("New version".to_string()),
        };

        let file_path = Path::new("/tmp/test.pdf");

        // Act
        let result = create_term_of_use_use_case(
            &repository,
            &storage,
            &cache,
            dto,
            file_path,
            "application/pdf",
        )
        .await;

        // Assert
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.version, 4); // Incremented from 3
    }

    #[tokio::test]
    async fn test_create_term_of_use_storage_failure() {
        // Arrange
        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        let mut storage = MockStorageService::new();
        storage
            .expect_upload_file()
            .returning(|_, _| Err(TermsOfUseError::InternalServerError));

        let cache = MockCacheService::new();

        let dto = CreateTermOfUseDTO {
            group: "privacy-policy".to_string(),
            info: None,
        };

        let file_path = Path::new("/tmp/test.pdf");

        // Act
        let result = create_term_of_use_use_case(
            &repository,
            &storage,
            &cache,
            dto,
            file_path,
            "application/pdf",
        )
        .await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), TermsOfUseError::InternalServerError);
    }

    #[tokio::test]
    async fn test_create_term_of_use_repository_failure_deletes_uploaded_file() {
        // Arrange
        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        repository
            .expect_create_term()
            .returning(|_| Err(TermsOfUseError::InternalServerError));

        let mut storage = MockStorageService::new();
        storage
            .expect_upload_file()
            .returning(|_, _| Ok("uploads/test-file.pdf".to_string()));

        storage
            .expect_delete_file()
            .with(eq("uploads/test-file.pdf"))
            .times(1)
            .returning(|_| Ok(()));

        let cache = MockCacheService::new();

        let dto = CreateTermOfUseDTO {
            group: "privacy-policy".to_string(),
            info: None,
        };

        let file_path = Path::new("/tmp/test.pdf");

        // Act
        let result = create_term_of_use_use_case(
            &repository,
            &storage,
            &cache,
            dto,
            file_path,
            "application/pdf",
        )
        .await;

        // Assert
        assert!(result.is_err());
        // Mock will verify delete_file was called
    }

    #[tokio::test]
    async fn test_create_term_of_use_without_info() {
        // Arrange
        let mut repository = MockTermRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .returning(|_| Ok(None));

        repository.expect_create_term().returning(|mut term| {
            term.id = 100;
            Ok(term)
        });

        let mut storage = MockStorageService::new();
        storage
            .expect_upload_file()
            .returning(|_, _| Ok("uploads/test-file.pdf".to_string()));

        storage
            .expect_get_file_url()
            .returning(|_| Ok("https://storage.example.com/test-file.pdf".to_string()));

        let mut cache = MockCacheService::new();
        cache
            .expect_invalidate_cache_for_group()
            .returning(|_| Ok(()));

        let dto = CreateTermOfUseDTO {
            group: "terms-of-service".to_string(),
            info: None,
        };

        let file_path = Path::new("/tmp/test.pdf");

        // Act
        let result = create_term_of_use_use_case(
            &repository,
            &storage,
            &cache,
            dto,
            file_path,
            "application/pdf",
        )
        .await;

        // Assert
        assert!(result.is_ok());
        let term = result.unwrap();
        assert_eq!(term.group, "terms-of-service");
        assert_eq!(term.info, None);
    }
}
