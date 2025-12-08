use tonic::Status;

use crate::{
    domain::{entities::TermOfUse, errors::TermsOfUseError},
    inbound::grpc::{CreateTermResponse, get_latest_terms_response::TermContent},
};

impl From<TermsOfUseError> for Status {
    fn from(value: TermsOfUseError) -> Self {
        match value {
            TermsOfUseError::InternalServerError => Status::internal("Internal server error"),
            TermsOfUseError::NotFound => Status::not_found("Terms of use not found"),
        }
    }
}

impl From<TermOfUse> for TermContent {
    fn from(term: TermOfUse) -> Self {
        TermContent {
            id: term.id,
            group: term.group,
            url: term.url,
            info: term.info,
        }
    }
}

impl From<TermOfUse> for CreateTermResponse {
    fn from(term: TermOfUse) -> Self {
        CreateTermResponse {
            id: term.id,
            group: term.group,
            url: term.url,
            info: term.info,
        }
    }
}
