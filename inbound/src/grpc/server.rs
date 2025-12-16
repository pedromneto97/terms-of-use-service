use domain::{
    dto::CreateTermOfUseDTO,
    use_cases::{
        create_term_of_use_use_case, create_user_agreement_use_case, get_latest_term_use_case,
        has_user_agreed_to_term_use_case,
    },
};
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info};

use crate::{
    config::Config,
    grpc::{
        CreateConsentRequest, CreateTermRequest, CreateTermResponse, GetLatestTermsRequest,
        GetLatestTermsResponse, HasConsentResponse, HasConsentedRequest,
        create_term_request::{CreateTermContent, CreateTermData},
        file_upload,
        get_latest_terms_response::TermOfUseContent,
        mapper::ToStatus,
        terms_of_use_service_server::TermsOfUseService,
    },
};

pub struct GrpcService {
    config: Config,
}

impl GrpcService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[tonic::async_trait]
impl TermsOfUseService for GrpcService {
    #[tracing::instrument(skip(self, request))]
    async fn has_consent(
        &self,
        request: Request<HasConsentedRequest>,
    ) -> Result<Response<HasConsentResponse>, Status> {
        let request = request.into_inner();

        let result = has_user_agreed_to_term_use_case(
            self.config.repository.as_ref(),
            self.config.cache.as_ref(),
            request.user_id,
            &request.group,
        )
        .await
        .map_err(|e| e.to_status())?;

        Ok(Response::new(HasConsentResponse {
            has_consented: result,
        }))
    }

    #[tracing::instrument(skip(self, request))]
    async fn get_latest_terms(
        &self,
        request: Request<GetLatestTermsRequest>,
    ) -> Result<Response<GetLatestTermsResponse>, Status> {
        let request = request.into_inner();

        let terms = get_latest_term_use_case(
            self.config.repository.as_ref(),
            self.config.cache.as_ref(),
            self.config.storage.as_ref(),
            &request.group,
        )
        .await
        .map_err(|e| e.to_status())?;

        Ok(Response::new(GetLatestTermsResponse {
            term_of_use_content: Some(match request.only_url {
                true => TermOfUseContent::Url(terms.url),
                false => TermOfUseContent::Term(terms.into()),
            }),
        }))
    }

    #[tracing::instrument(skip(self, request))]
    async fn create_consent(
        &self,
        request: Request<CreateConsentRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        create_user_agreement_use_case(
            self.config.repository.as_ref(),
            self.config.cache.as_ref(),
            self.config.publisher.as_ref(),
            request.user_id,
            request.term_id,
        )
        .await
        .map_err(|e| e.to_status())?;

        Ok(Response::new(()))
    }

    #[tracing::instrument(skip(self, request))]
    async fn create_term(
        &self,
        request: Request<Streaming<CreateTermRequest>>,
    ) -> Result<Response<CreateTermResponse>, Status> {
        let mut create_term_data: Option<CreateTermData> = None;
        let (mut file, file_path) = file_upload::create_temp_file().await?;

        let mut stream = request.into_inner();

        while let Some(request_data) = stream.message().await? {
            if let Some(content) = request_data.create_term_content {
                match content {
                    CreateTermContent::Data(data) => {
                        info!(
                            "Received term data: group={}, info={:?}, content_type={}, content_size={}",
                            data.group, data.info, data.content_type, data.content_size
                        );

                        create_term_data = Some(data);
                    }
                    CreateTermContent::Chunk(chunk) => {
                        debug!("Received term chunk of size: {}", chunk.len());

                        file.write_all(&chunk).await.map_err(|e| {
                            error!("Failed to write chunk to temp file: {e}");

                            Status::internal(format!("Failed to write chunk to temp file: {e}"))
                        })?;
                    }
                }
            }
        }
        let data = match create_term_data {
            Some(data) => data,
            None => {
                return Err(Status::invalid_argument("No term data provided"));
            }
        };

        let term = create_term_of_use_use_case(
            self.config.repository.as_ref(),
            self.config.storage.as_ref(),
            self.config.cache.as_ref(),
            CreateTermOfUseDTO {
                group: data.group,
                info: data.info,
            },
            &file_path,
            &data.content_type,
        )
        .await
        .map_err(|e| e.to_status())?;

        Ok(Response::new(CreateTermResponse::from(term)))
    }
}
