use actix_multipart::form::MultipartForm;
use actix_web::{
    HttpResponse, get, post,
    web::{self, Path},
};
use domain::use_cases::{
    create_term_of_use_use_case, create_user_agreement_use_case, get_latest_term_use_case,
    has_user_agreed_to_term_use_case,
};

use crate::{
    actix::{
        error::response::ProblemDetails,
        v1::{
            payload::{CreateAgreementPayload, CreateTermForm, GetLatestTermPayload},
            response::{HasConsentedResponse, TermOfUseResponse, TermOfUseUrlResponse},
        },
    },
    config::Config,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/terms-of-use")
            .service(has_user_consented_to_latest_term)
            .service(create_agreement)
            .service(create_term_of_use)
            .service(get_latest_term_for_group),
    );
}

#[tracing::instrument(skip(config, group))]
#[get("/has-consent/{group}/{user_id}")]
async fn has_user_consented_to_latest_term(
    group: Path<(String, i32)>,
    config: web::Data<Config>,
) -> Result<HttpResponse, ProblemDetails> {
    let (group, user_id) = group.into_inner();

    let term = has_user_agreed_to_term_use_case(
        config.repository.as_ref(),
        config.cache.as_ref(),
        user_id,
        &group,
    )
    .await?;

    Ok(HttpResponse::Ok().json(HasConsentedResponse {
        has_consented: term,
    }))
}

#[tracing::instrument(skip(config, body))]
#[post("/agreements")]
async fn create_agreement(
    config: web::Data<Config>,
    body: web::Json<CreateAgreementPayload>,
) -> Result<HttpResponse, ProblemDetails> {
    let CreateAgreementPayload { user_id, term_id } = body.into_inner();

    create_user_agreement_use_case(
        config.repository.as_ref(),
        config.cache.as_ref(),
        config.publisher.as_ref(),
        user_id,
        term_id,
    )
    .await?;

    Ok(HttpResponse::Created().finish())
}

#[tracing::instrument(skip(config, body))]
#[post("/")]
async fn create_term_of_use(
    config: web::Data<Config>,
    MultipartForm(body): MultipartForm<CreateTermForm>,
) -> Result<HttpResponse, ProblemDetails> {
    let CreateTermForm { file, data } = body;

    let content_type = file
        .content_type
        .map(|ct| ct.to_string())
        .unwrap_or_default();

    if content_type.is_empty() || content_type != "application/pdf" {
        return Err(
            ProblemDetails::bad_request().with_detail("Term of use file must be a valid PDF")
        );
    }

    create_term_of_use_use_case(
        config.repository.as_ref(),
        config.storage.as_ref(),
        config.cache.as_ref(),
        data.into_inner().into(),
        file.file.path(),
        &content_type,
    )
    .await?;

    Ok(HttpResponse::Created().finish())
}

#[tracing::instrument(skip(config, group, payload))]
#[get("/{group}")]
async fn get_latest_term_for_group(
    group: Path<String>,
    payload: web::Query<GetLatestTermPayload>,
    config: web::Data<Config>,
) -> Result<HttpResponse, ProblemDetails> {
    let term = get_latest_term_use_case(
        config.repository.as_ref(),
        config.cache.as_ref(),
        config.storage.as_ref(),
        &group,
    )
    .await?;

    if payload.only_url {
        return Ok(HttpResponse::Ok().json(TermOfUseUrlResponse::from(term)));
    }

    Ok(HttpResponse::Ok().json(TermOfUseResponse::from(term)))
}

#[cfg(test)]
mod tests {
    use actix_web::{App, http::StatusCode, test, web};
    use chrono::Utc;
    use domain::entities::TermOfUse;
    use mockall::predicate::eq;
    use serde_json::Value;
    use std::sync::Arc;

    use crate::{
        Config,
        actix::v1::{controller::configure, payload::CreateAgreementPayload},
        mocks::*,
    };

    fn build_config(
        repository: MockDatabaseRepository,
        cache: MockCacheService,
        storage: MockStorageService,
        publisher: MockPublisherService,
    ) -> Config {
        Config {
            repository: Arc::new(repository),
            cache: Arc::new(cache),
            storage: Arc::new(storage),
            publisher: Arc::new(publisher),
        }
    }

    fn sample_term(group: &str) -> TermOfUse {
        TermOfUse {
            id: 1,
            group: group.to_string(),
            url: "stored/path.pdf".to_string(),
            version: 1,
            info: Some("info".to_string()),
            created_at: Utc::now().naive_utc(),
        }
    }

    #[actix_web::test]
    async fn has_user_consented_reads_cache_first() {
        let mut repository = MockDatabaseRepository::new();
        repository.expect_get_latest_term_for_group().times(0);
        repository.expect_has_user_agreed_to_term().times(0);

        let mut cache = MockCacheService::new();
        cache
            .expect_find_user_agreement()
            .with(eq(7), eq("alpha"))
            .returning(|_, _| Ok(Some(true)));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_config(
                    repository,
                    cache,
                    MockStorageService::new(),
                    MockPublisherService::new(),
                )))
                .configure(configure),
        )
        .await;

        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/v1/terms-of-use/has-consent/alpha/7")
                .to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = test::read_body(response).await;
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["hasConsented"], true);
    }

    #[actix_web::test]
    async fn create_agreement_publishes_and_caches() {
        let mut repository = MockDatabaseRepository::new();
        repository
            .expect_get_term_by_id()
            .with(eq(3))
            .returning(|_| Ok(Some(sample_term("legal"))));
        repository
            .expect_create_user_agreement()
            .with(eq(42), eq(3))
            .returning(|_, _| Ok(()));

        let mut cache = MockCacheService::new();
        cache
            .expect_store_user_agreement()
            .with(eq(42), eq("legal"), eq(true))
            .returning(|_, _, _| Ok(()));

        let mut publisher = MockPublisherService::new();
        publisher.expect_publish_agreement().returning(|_| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_config(
                    repository,
                    cache,
                    MockStorageService::new(),
                    publisher,
                )))
                .configure(configure),
        )
        .await;

        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/terms-of-use/agreements")
                .set_json(&CreateAgreementPayload {
                    user_id: 42,
                    term_id: 3,
                })
                .to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn create_term_of_use_accepts_pdf_upload() {
        let mut repository = MockDatabaseRepository::new();
        repository
            .expect_get_latest_term_for_group()
            .with(eq("legal"))
            .returning(|_| Ok(None));
        repository.expect_create_term().returning(|mut term| {
            term.id = 10;
            Ok(term)
        });

        let mut cache = MockCacheService::new();
        cache
            .expect_invalidate_cache_for_group()
            .with(eq("legal"))
            .returning(|_| Ok(()));

        let mut storage = MockStorageService::new();
        storage
            .expect_upload_file()
            .withf(|_, content_type| content_type == "application/pdf")
            .returning(|_, _| Ok("stored/path.pdf".to_string()));
        storage
            .expect_get_file_url()
            .with(eq("stored/path.pdf"))
            .returning(|_| Ok("https://files/terms.pdf".to_string()));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_config(
                    repository,
                    cache,
                    storage,
                    MockPublisherService::new(),
                )))
                .configure(configure),
        )
        .await;

        let boundary = "boundary123";
        let payload = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"example/sample.pdf\"\r\nContent-Type: application/pdf\r\n\r\nPDF\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"data\"\r\nContent-Type: application/json\r\n\r\n{{\"group\":\"legal\",\"info\":\"v1\"}}\r\n--{boundary}--\r\n"
        );

        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/terms-of-use/")
                .insert_header((
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                ))
                .set_payload(payload)
                .to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn create_term_of_use_rejects_non_pdf() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_config(
                    MockDatabaseRepository::new(),
                    MockCacheService::new(),
                    MockStorageService::new(),
                    MockPublisherService::new(),
                )))
                .configure(configure),
        )
        .await;

        let boundary = "boundary456";
        let payload = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"terms.txt\"\r\nContent-Type: text/plain\r\n\r\nnot pdf\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"data\"\r\nContent-Type: application/json\r\n\r\n{{\"group\":\"legal\"}}\r\n--{boundary}--\r\n"
        );

        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/terms-of-use/")
                .insert_header((
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                ))
                .set_payload(payload)
                .to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(response).await;
        let problem: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(problem["detail"], "Term of use file must be a valid PDF");
    }

    #[actix_web::test]
    async fn get_latest_term_returns_full_payload() {
        let term = sample_term("finance");

        let mut repository = MockDatabaseRepository::new();
        repository.expect_get_latest_term_for_group().times(0);

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .with(eq("finance"))
            .returning(move |_| Ok(Some(term.clone())));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_config(
                    repository,
                    cache,
                    MockStorageService::new(),
                    MockPublisherService::new(),
                )))
                .configure(configure),
        )
        .await;

        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/v1/terms-of-use/finance")
                .to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = test::read_body(response).await;
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["url"], "stored/path.pdf");
        assert_eq!(payload["group"], "finance");
    }

    #[actix_web::test]
    async fn get_latest_term_returns_url_only() {
        let term = sample_term("hr");

        let mut cache = MockCacheService::new();
        cache
            .expect_get_latest_term_for_group()
            .with(eq("hr"))
            .returning(move |_| Ok(Some(term.clone())));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(build_config(
                    MockDatabaseRepository::new(),
                    cache,
                    MockStorageService::new(),
                    MockPublisherService::new(),
                )))
                .configure(configure),
        )
        .await;

        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/v1/terms-of-use/hr?only_url=true")
                .to_request(),
        )
        .await;

        let body = test::read_body(response).await;
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["url"], "stored/path.pdf");
    }
}
