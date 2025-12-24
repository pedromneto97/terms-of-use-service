use tokio_stream::StreamExt;
use tonic::Request;
use tonic_health::pb::{
    HealthCheckRequest, health_check_response::ServingStatus as PbServingStatus,
    health_server::Health,
};

use crate::{
    grpc::server::GrpcService,
    grpc::tests::create_test_config,
    mocks::{MockCacheService, MockDatabaseRepository, MockPublisherService, MockStorageService},
};

#[tokio::test]
async fn test_health_check_check_serving_when_all_ok() {
    let mut repo = MockDatabaseRepository::new();
    repo.expect_ping().times(1).returning(|| Ok(()));

    let mut cache = MockCacheService::new();
    cache.expect_ping().times(1).returning(|| Ok(()));

    let mut storage = MockStorageService::new();
    storage.expect_ping().times(1).returning(|| Ok(()));

    let mut publisher = MockPublisherService::new();
    publisher.expect_ping().times(1).returning(|| Ok(()));

    let config = create_test_config(Some(repo), Some(cache), Some(storage), Some(publisher));
    let service = GrpcService::new(config);

    let req = Request::new(HealthCheckRequest {
        service: String::new(),
    });
    let res = service.check(req).await.unwrap().into_inner();

    assert_eq!(res.status, PbServingStatus::Serving as i32);
}

#[tokio::test]
async fn test_health_check_check_not_serving_when_any_fail() {
    let mut repo = MockDatabaseRepository::new();
    repo.expect_ping().times(1).returning(|| Ok(()));

    let mut cache = MockCacheService::new();
    cache.expect_ping().times(1).returning(|| Ok(()));

    let mut storage = MockStorageService::new();
    storage
        .expect_ping()
        .times(1)
        .returning(|| Err(domain::errors::TermsOfUseError::InternalServerError));

    let mut publisher = MockPublisherService::new();
    publisher.expect_ping().times(1).returning(|| Ok(()));

    let config = create_test_config(Some(repo), Some(cache), Some(storage), Some(publisher));
    let service = GrpcService::new(config);

    let req = Request::new(HealthCheckRequest {
        service: String::new(),
    });

    let res = service.check(req).await.unwrap().into_inner();

    assert_eq!(res.status, PbServingStatus::NotServing as i32);
}

#[tokio::test]
async fn test_health_check_watch_initial_unknown() {
    let mut repository = MockDatabaseRepository::new();
    repository.expect_ping().times(1).returning(|| Ok(()));

    let mut cache = MockCacheService::new();
    cache.expect_ping().times(1).returning(|| Ok(()));

    let mut storage = MockStorageService::new();
    storage
        .expect_ping()
        .times(1)
        .returning(|| Err(domain::errors::TermsOfUseError::InternalServerError));

    let mut publisher = MockPublisherService::new();
    publisher.expect_ping().times(1).returning(|| Ok(()));

    let config = create_test_config(
        Some(repository),
        Some(cache),
        Some(storage),
        Some(publisher),
    );
    let service = GrpcService::new(config);

    let req = Request::new(HealthCheckRequest {
        service: String::new(),
    });
    let mut stream = service.watch(req).await.unwrap().into_inner();

    let first = stream.next().await.expect("stream item").expect("ok item");
    assert_eq!(first.status, PbServingStatus::Unknown as i32);

    let second = stream.next().await.expect("stream item").expect("ok item");
    assert_eq!(second.status, PbServingStatus::NotServing as i32);
}
