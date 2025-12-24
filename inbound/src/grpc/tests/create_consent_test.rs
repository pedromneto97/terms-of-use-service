use domain::{entities::TermOfUse, errors::TermsOfUseError};
use mockall::predicate::*;
use tonic::{Code, Request};

use crate::{
    grpc::{
        CreateConsentRequest, server::GrpcService, terms_of_use_service_server::TermsOfUseService,
        tests::create_test_config,
    },
    mocks::{MockCacheService, MockDatabaseRepository, MockPublisherService},
};

#[tokio::test]
async fn test_create_consent_success() {
    const USER_ID: i32 = 100;
    const TERM_ID: i32 = 5;
    const GROUP: &str = "privacy-policy";

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_term_by_id()
        .with(eq(TERM_ID))
        .times(1)
        .returning(move |_| {
            Ok(Some(TermOfUse {
                id: TERM_ID,
                group: GROUP.to_string(),
                version: 1,
                url: "uploads/privacy-v1.pdf".to_string(),
                created_at: chrono::Utc::now().naive_utc(),
                info: None,
            }))
        });
    mock_repo
        .expect_create_user_agreement()
        .with(eq(USER_ID), eq(TERM_ID))
        .times(1)
        .returning(|_, _| Ok(()));

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_store_user_agreement()
        .with(eq(USER_ID), eq(GROUP), eq(true))
        .times(1)
        .returning(|_, _, _| Ok(()));

    let mut mock_publisher = MockPublisherService::new();
    mock_publisher
        .expect_publish_agreement()
        .times(1)
        .returning(|_| Ok(()));

    let config = create_test_config(
        Some(mock_repo),
        Some(mock_cache),
        None,
        Some(mock_publisher),
    );
    let service = GrpcService::new(config);

    let request = Request::new(CreateConsentRequest {
        user_id: USER_ID,
        term_id: TERM_ID,
    });

    let response = service.create_consent(request).await;

    assert!(response.is_ok());
}

#[tokio::test]
async fn test_create_consent_term_not_found() {
    const USER_ID: i32 = 200;
    const TERM_ID: i32 = 999;

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_term_by_id()
        .with(eq(TERM_ID))
        .times(1)
        .returning(|_| Err(TermsOfUseError::NotFound));

    let config = create_test_config(Some(mock_repo), None, None, None);
    let service = GrpcService::new(config);

    let request = Request::new(CreateConsentRequest {
        user_id: USER_ID,
        term_id: TERM_ID,
    });

    let response = service.create_consent(request).await;

    assert!(response.is_err());
    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);
}
