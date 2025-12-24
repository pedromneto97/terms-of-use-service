use chrono::Utc;
use domain::{entities::TermOfUse, errors::TermsOfUseError};
use mockall::predicate::eq;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    grpc::{
        CreateTermRequest,
        create_term_request::{CreateTermContent, CreateTermData},
        server::GrpcService,
        terms_of_use_service_client::TermsOfUseServiceClient,
        terms_of_use_service_server::TermsOfUseServiceServer,
        tests::create_test_config,
    },
    mocks::{MockCacheService, MockDatabaseRepository, MockStorageService},
};

async fn spawn_test_server(service: GrpcService) -> (String, oneshot::Sender<()>) {
    let (tx, rx) = oneshot::channel();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
        if let Err(e) = Server::builder()
            .add_service(TermsOfUseServiceServer::new(service))
            .serve_with_incoming_shutdown(incoming, async {
                rx.await.ok();
            })
            .await
        {
            eprintln!("gRPC test server failed: {e}");
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (url, tx)
}

#[tokio::test]
async fn test_create_term_client_success() {
    const GROUP: &str = "privacy-policy";
    const INFO: &str = "Privacy policy v1";
    const CONTENT_TYPE: &str = "application/pdf";
    const CONTENT: &[u8] = b"PDF content here";
    const CONTENT_SIZE: u64 = CONTENT.len() as u64;
    const TERM_ID: i32 = 100;

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(|_| Ok(None));
    mock_repo.expect_create_term().times(1).returning(move |_| {
        Ok(TermOfUse {
            id: TERM_ID,
            group: GROUP.to_string(),
            version: 1,
            url: "uploads/privacy-v1.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: Some(INFO.to_string()),
        })
    });

    let mut mock_storage = MockStorageService::new();
    mock_storage
        .expect_upload_file()
        .times(1)
        .returning(|_, _| Ok("uploads/privacy-v1.pdf".to_string()));
    mock_storage
        .expect_get_file_url()
        .times(1)
        .returning(|_| Ok("https://storage.example.com/uploads/privacy-v1.pdf".to_string()));

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_invalidate_cache_for_group()
        .times(1)
        .returning(|_| Ok(()));

    let config = create_test_config(Some(mock_repo), Some(mock_cache), Some(mock_storage), None);
    let service = GrpcService::new(config);
    let (url, shutdown) = spawn_test_server(service).await;

    let mut client = TermsOfUseServiceClient::connect(url).await.unwrap();

    let messages = vec![
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Data(CreateTermData {
                group: GROUP.to_string(),
                info: Some(INFO.to_string()),
                content_type: CONTENT_TYPE.to_string(),
                content_size: CONTENT_SIZE,
            })),
        },
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Chunk(CONTENT.to_vec())),
        },
    ];
    let response = client.create_term(tokio_stream::iter(messages)).await;

    let response = response.unwrap().into_inner();
    assert_eq!(response.id, TERM_ID);
    assert_eq!(response.group, GROUP);
    assert_eq!(
        response.url,
        "https://storage.example.com/uploads/privacy-v1.pdf"
    );
    assert_eq!(response.info, Some(INFO.to_string()));

    // Cleanup
    shutdown.send(()).ok();
}

#[tokio::test]
async fn test_create_term_client_multiple_chunks() {
    const GROUP: &str = "terms-of-service";
    const CONTENT_TYPE: &str = "text/plain";
    const CHUNK1: &[u8] = b"First chunk";
    const CHUNK2: &[u8] = b"Second chunk";
    const CHUNK3: &[u8] = b"Third chunk";
    const CONTENT_SIZE: u64 = (CHUNK1.len() + CHUNK2.len() + CHUNK3.len()) as u64;
    const TERM_ID: i32 = 200;

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(|_| Ok(None));

    mock_repo.expect_create_term().times(1).returning(move |_| {
        Ok(TermOfUse {
            id: TERM_ID,
            group: GROUP.to_string(),
            version: 1,
            url: "uploads/tos-v1.txt".to_string(),
            created_at: Utc::now().naive_utc(),
            info: None,
        })
    });

    let mut mock_storage = MockStorageService::new();
    mock_storage
        .expect_upload_file()
        .times(1)
        .returning(|_, _| Ok("uploads/tos-v1.txt".to_string()));

    mock_storage
        .expect_get_file_url()
        .times(1)
        .returning(|_| Ok("https://storage.example.com/uploads/tos-v1.txt".to_string()));

    let mut mock_cache = MockCacheService::new();
    mock_cache
        .expect_invalidate_cache_for_group()
        .times(1)
        .returning(|_| Ok(()));

    let config = create_test_config(Some(mock_repo), Some(mock_cache), Some(mock_storage), None);
    let service = GrpcService::new(config);
    let (url, shutdown) = spawn_test_server(service).await;

    let mut client = TermsOfUseServiceClient::connect(url).await.unwrap();

    let messages = vec![
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Data(CreateTermData {
                group: GROUP.to_string(),
                info: None,
                content_type: CONTENT_TYPE.to_string(),
                content_size: CONTENT_SIZE,
            })),
        },
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Chunk(CHUNK1.to_vec())),
        },
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Chunk(CHUNK2.to_vec())),
        },
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Chunk(CHUNK3.to_vec())),
        },
    ];

    let stream = tokio_stream::iter(messages);

    let response = client.create_term(stream).await;

    let response = response.unwrap().into_inner();
    assert_eq!(response.id, TERM_ID);
    assert_eq!(response.group, GROUP);

    shutdown.send(()).ok();
}

#[tokio::test]
async fn test_create_term_client_no_data_error() {
    let config = create_test_config(None, None, None, None);
    let service = GrpcService::new(config);
    let (url, shutdown) = spawn_test_server(service).await;

    let mut client = TermsOfUseServiceClient::connect(url).await.unwrap();

    let messages = vec![CreateTermRequest {
        create_term_content: Some(CreateTermContent::Chunk(b"orphan chunk".to_vec())),
    }];

    let stream = tokio_stream::iter(messages);

    let response = client.create_term(stream).await;

    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("No term data provided"));

    shutdown.send(()).ok();
}

#[tokio::test]
async fn test_create_term_client_use_case_error() {
    const GROUP: &str = "error-group";
    const CONTENT_TYPE: &str = "application/pdf";
    const CONTENT: &[u8] = b"PDF content";
    const CONTENT_SIZE: u64 = CONTENT.len() as u64;

    let mut mock_repo = MockDatabaseRepository::new();
    mock_repo
        .expect_get_latest_term_for_group()
        .with(eq(GROUP))
        .times(1)
        .returning(|_| Ok(None));

    mock_repo
        .expect_create_term()
        .times(1)
        .returning(|_| Err(TermsOfUseError::InternalServerError));

    let mut mock_storage = MockStorageService::new();
    mock_storage
        .expect_upload_file()
        .times(1)
        .returning(|_, _| Ok("uploads/error.pdf".to_string()));
    mock_storage
        .expect_delete_file()
        .times(1)
        .returning(|_| Ok(()));

    let config = create_test_config(Some(mock_repo), None, Some(mock_storage), None);
    let service = GrpcService::new(config);
    let (url, shutdown) = spawn_test_server(service).await;

    let mut client = TermsOfUseServiceClient::connect(url).await.unwrap();

    let messages = vec![
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Data(CreateTermData {
                group: GROUP.to_string(),
                info: None,
                content_type: CONTENT_TYPE.to_string(),
                content_size: CONTENT_SIZE,
            })),
        },
        CreateTermRequest {
            create_term_content: Some(CreateTermContent::Chunk(CONTENT.to_vec())),
        },
    ];

    let response = client.create_term(tokio_stream::iter(messages)).await;

    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Internal);

    shutdown.send(()).ok();
}
