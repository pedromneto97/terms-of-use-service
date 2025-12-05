use actix_multipart::form::MultipartForm;
use actix_web::{
    HttpResponse, get, post,
    web::{self, Path},
};

use crate::{
    core::Config,
    domain::use_cases::{
        create_term_of_use_use_case, create_user_agreement_use_case, get_latest_term_use_case,
        has_user_agreed_to_term_use_case,
    },
    inbound::actix::{
        error::response::ProblemDetails,
        v1::{
            payload::{CreateAgreementPayload, CreateTermForm, GetLatestTermPayload},
            response::{HasConsentedResponse, TermOfUseResponse, TermOfUseUrlResponse},
        },
    },
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

#[get("/has-consent/{group}/{user_id}")]
async fn has_user_consented_to_latest_term(
    group: Path<(String, i32)>,
    config: web::Data<Config>,
) -> Result<HttpResponse, ProblemDetails> {
    let (group, user_id) = group.into_inner();

    let term = has_user_agreed_to_term_use_case(&config.repository, &config.cache, user_id, &group)
        .await?;

    Ok(HttpResponse::Ok().json(HasConsentedResponse {
        has_consented: term,
    }))
}

#[post("/agreements")]
async fn create_agreement(
    config: web::Data<Config>,
    body: web::Json<CreateAgreementPayload>,
) -> Result<HttpResponse, ProblemDetails> {
    let CreateAgreementPayload { user_id, term_id } = body.into_inner();

    create_user_agreement_use_case(&config.repository, &config.cache, user_id, term_id).await?;

    Ok(HttpResponse::Created().finish())
}

#[post("/")]
async fn create_term_of_use(
    config: web::Data<Config>,
    MultipartForm(body): MultipartForm<CreateTermForm>,
) -> Result<HttpResponse, ProblemDetails> {
    let CreateTermForm { file, data } = body;

    create_term_of_use_use_case(
        &config.repository,
        &config.storage,
        &config.cache,
        data.into_inner().into(),
        file.file.path(),
    )
    .await?;

    Ok(HttpResponse::Created().finish())
}

#[get("/{group}")]
async fn get_latest_term_for_group(
    group: Path<String>,
    payload: web::Query<GetLatestTermPayload>,
    config: web::Data<Config>,
) -> Result<HttpResponse, ProblemDetails> {
    let term = get_latest_term_use_case(&config.repository, &config.cache, &config.storage, &group)
        .await?;

    if payload.only_url {
        return Ok(HttpResponse::Ok().json(TermOfUseUrlResponse::from(term)));
    }

    Ok(HttpResponse::Ok().json(TermOfUseResponse::from(term)))
}
