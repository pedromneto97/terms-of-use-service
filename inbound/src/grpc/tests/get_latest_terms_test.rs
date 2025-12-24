use chrono::Utc;
use domain::errors::TermsOfUseError;
use mockall::predicate::*;
use tonic::{Code, Request};

use crate::{
    grpc::{
        GetLatestTermsRequest, get_latest_terms_response::TermOfUseContent, server::GrpcService,
        terms_of_use_service_server::TermsOfUseService, tests::create_test_config,
    },
    mocks::{MockCacheService, MockDatabaseRepository},
};

#[tokio::test]
async fn test_get_latest_terms_success_full_content() {
    const GROUP: &str = "privacy-policy";
    const TERM_ID: i32 = 5;
    const TERM_URL: &str = "uploads/privacy-v1.pdf";
    const TERM_INFO: &str = "Privacy policy v1";

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(move |_| {
            Ok(Some(domain::entities::TermOfUse {
                id: TERM_ID,
                group: GROUP.to_string(),
                version: 1,
                url: TERM_URL.to_string(),
                created_at: Utc::now().naive_utc(),
                info: Some(TERM_INFO.to_string()),
            }))
        });

    let config = create_test_config(None, Some(mock_cache), None, None);
    let service = GrpcService::new(config);

    let request = Request::new(GetLatestTermsRequest {
        group: GROUP.to_string(),
        only_url: false,
    });

    let response = service.get_latest_terms(request).await;

    let response = response.unwrap().into_inner();
    assert!(response.term_of_use_content.is_some());

    match response.term_of_use_content.unwrap() {
        TermOfUseContent::Term(term) => {
            assert_eq!(term.id, TERM_ID);
            assert_eq!(term.group, GROUP);
            assert_eq!(term.url, TERM_URL);
            assert_eq!(term.info, Some(TERM_INFO.to_string()));
        }
        TermOfUseContent::Url(_) => panic!("Expected Term, got Url"),
    }
}

#[tokio::test]
async fn test_get_latest_terms_success_url_only() {
    const GROUP: &str = "terms-of-service";
    const TERM_ID: i32 = 12;
    const TERM_URL: &str = "uploads/tos-v2.pdf";

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(move |_| {
            Ok(Some(domain::entities::TermOfUse {
                id: TERM_ID,
                group: GROUP.to_string(),
                version: 2,
                url: TERM_URL.to_string(),
                created_at: Utc::now().naive_utc(),
                info: None,
            }))
        });

    let config = create_test_config(None, Some(mock_cache), None, None);
    let service = GrpcService::new(config);

    let request = Request::new(GetLatestTermsRequest {
        group: GROUP.to_string(),
        only_url: true,
    });

    let response = service.get_latest_terms(request).await;

    let response = response.unwrap().into_inner();

    match response.term_of_use_content.unwrap() {
        TermOfUseContent::Url(url) => {
            assert_eq!(url, TERM_URL);
        }
        TermOfUseContent::Term(_) => panic!("Expected Url, got Term"),
    }
}

#[tokio::test]
async fn test_get_latest_terms_not_found_error() {
    const GROUP: &str = "non-existent";

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(|_| Ok(None));

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(|_| Err(TermsOfUseError::NotFound));

    let config = create_test_config(Some(mock_repo), Some(mock_cache), None, None);
    let service = GrpcService::new(config);

    let request = Request::new(GetLatestTermsRequest {
        group: GROUP.to_string(),
        only_url: false,
    });

    let response = service.get_latest_terms(request).await;

    let status = response.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);
}
