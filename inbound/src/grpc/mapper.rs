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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::{entities::TermOfUse, errors::TermsOfUseError};
    use tonic::Code;

    use crate::grpc::{
        CreateTermResponse, get_latest_terms_response::TermContent, mapper::ToStatus,
    };

    #[test]
    fn test_to_status_not_found() {
        let error = TermsOfUseError::NotFound;

        let status = error.to_status();

        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("not found"));
    }

    #[test]
    fn test_to_status_internal_server_error() {
        let error = TermsOfUseError::InternalServerError;

        let status = error.to_status();

        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("Internal server error"));
    }

    #[test]
    fn test_term_of_use_to_term_content() {
        let term = TermOfUse {
            id: 42,
            group: "privacy-policy".to_string(),
            version: 3,
            url: "uploads/privacy-v3.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: Some("Latest privacy policy".to_string()),
        };

        let term_content: TermContent = term.clone().into();

        assert_eq!(term_content.id, term.id);
        assert_eq!(term_content.group, term.group);
        assert_eq!(term_content.url, term.url);
        assert_eq!(term_content.info, term.info);
    }

    #[test]
    fn test_term_of_use_to_create_term_response() {
        let term = TermOfUse {
            id: 99,
            group: "cookie-policy".to_string(),
            version: 2,
            url: "uploads/cookie-v2.pdf".to_string(),
            created_at: Utc::now().naive_utc(),
            info: Some("Updated cookie policy".to_string()),
        };

        let response: CreateTermResponse = term.clone().into();

        assert_eq!(response.id, term.id);
        assert_eq!(response.group, term.group);
        assert_eq!(response.url, term.url);
        assert_eq!(response.info, term.info);
    }
}
