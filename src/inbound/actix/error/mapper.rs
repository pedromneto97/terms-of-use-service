use actix_web::{HttpResponse, ResponseError, http::StatusCode};

use crate::domain::errors::TermsOfUseError;

use super::response::ProblemDetails;

impl From<TermsOfUseError> for ProblemDetails {
    fn from(error: TermsOfUseError) -> Self {
        match error {
            TermsOfUseError::NotFound => {
                ProblemDetails::not_found().with_detail("The requested terms of use was not found.")
            }

            TermsOfUseError::InternalServerError => ProblemDetails::internal_server_error()
                .with_detail("An unexpected error occurred. Please try again later."),
        }
    }
}

impl ResponseError for ProblemDetails {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error_mapping() {
        let problem: ProblemDetails = TermsOfUseError::NotFound.into();

        assert_eq!(problem.problem_type, "about:blank");
        assert_eq!(problem.title, "Not Found");
        assert_eq!(problem.status, 404);
        assert_eq!(
            problem.detail,
            Some("The requested terms of use was not found.".to_string())
        );
    }

    #[test]
    fn test_internal_server_error_mapping() {
        let problem: ProblemDetails = TermsOfUseError::InternalServerError.into();

        assert_eq!(problem.problem_type, "about:blank");
        assert_eq!(problem.title, "Internal Server Error");
        assert_eq!(problem.status, 500);
        assert_eq!(
            problem.detail,
            Some("An unexpected error occurred. Please try again later.".to_string())
        );
    }
}
