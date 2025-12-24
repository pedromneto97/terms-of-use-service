use domain::errors::TermsOfUseError;
use mockall::predicate::*;
use tonic::{Code, Request};

use crate::{
    grpc::{
        HasConsentedRequest, server::GrpcService, terms_of_use_service_server::TermsOfUseService,
        tests::create_test_config,
    },
    mocks::{MockCacheService, MockDatabaseRepository},
};

#[tokio::test]
async fn test_has_consent_success_true() {
    const USER_ID: i32 = 123;
    const GROUP: &str = "privacy-policy";

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_find_user_agreement()
        .with(eq(USER_ID), eq(GROUP))
        .times(1)
        .returning(|_, _| Ok(Some(true)));

    let config = create_test_config(None, Some(mock_cache), None, None);
    let service = GrpcService::new(config);

    let request = Request::new(HasConsentedRequest {
        user_id: USER_ID,
        group: GROUP.to_string(),
    });

    let response = service.has_consent(request).await;

    let response = response.unwrap().into_inner();
    assert!(response.has_consented);
}

#[tokio::test]
async fn test_has_consent_not_found_error() {
    const USER_ID: i32 = 999;
    const GROUP: &str = "non-existent";

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_find_user_agreement()
        .with(eq(USER_ID), eq(GROUP))
        .times(1)
        .returning(|_, _| Ok(None));

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_latest_term_for_group()
        .with(eq(GROUP.to_string()))
        .times(1)
        .returning(|_| Err(TermsOfUseError::NotFound));

    let config = create_test_config(Some(mock_repo), Some(mock_cache), None, None);
    let service = GrpcService::new(config);

    let request = Request::new(HasConsentedRequest {
        user_id: USER_ID,
        group: GROUP.to_string(),
    });

    let response = service.has_consent(request).await;

    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);
}
