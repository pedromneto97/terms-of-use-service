use domain::{entities::TermOfUse, errors::TermsOfUseError};
use tonic::Status;

use crate::grpc::{CreateTermResponse, get_latest_terms_response::TermContent};

pub trait ToStatus {
    fn to_status(&self) -> Status;
}

impl ToStatus for TermsOfUseError {
    fn to_status(&self) -> Status {
        match self {
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
