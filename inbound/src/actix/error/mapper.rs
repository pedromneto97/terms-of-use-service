use actix_multipart::MultipartError;
use actix_web::{
    HttpRequest, HttpResponse, ResponseError,
    error::{JsonPayloadError, QueryPayloadError},
    http::StatusCode,
};
use domain::errors::TermsOfUseError;

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

pub fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let detail = match &err {
        JsonPayloadError::Deserialize(e) => format!("Invalid JSON: {e}"),
        JsonPayloadError::ContentType => "Content type must be application/json".to_string(),
        JsonPayloadError::Overflow { limit } => {
            format!("Payload size exceeded. Maximum size is {limit} bytes")
        }
        JsonPayloadError::Payload(payload) => format!("Payload error: {payload}"),
        _ => err.to_string(),
    };
    ProblemDetails::bad_request().with_detail(detail).into()
}

pub fn query_error_handler(err: QueryPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let detail = match &err {
        QueryPayloadError::Deserialize(e) => format!("Invalid query parameter: {e}"),
        _ => err.to_string(),
    };
    ProblemDetails::bad_request().with_detail(detail).into()
}

pub fn multipart_error_handler(err: MultipartError, _req: &HttpRequest) -> actix_web::Error {
    let detail = match err {
        MultipartError::ContentTypeMissing => {
            "Content-Type header is missing. Multipart requests require a Content-Type header."
                .to_string()
        }
        MultipartError::ContentTypeParse => "Failed to parse Content-Type header.".to_string(),
        MultipartError::ContentTypeIncompatible => {
            "Content-Type is not compatible with multipart/form-data.".to_string()
        }
        MultipartError::BoundaryMissing => {
            "Boundary parameter is missing from Content-Type header.".to_string()
        }
        MultipartError::ContentDispositionMissing => {
            "Content-Disposition header is missing in multipart field.".to_string()
        }
        MultipartError::ContentDispositionNameMissing => {
            "Name parameter is missing from Content-Disposition header.".to_string()
        }
        MultipartError::Nested => "Nested multipart is not supported.".to_string(),
        MultipartError::Incomplete => {
            "Multipart stream ended unexpectedly. The upload may be incomplete.".to_string()
        }
        MultipartError::Parse(parse_error) => {
            format!("Failed to parse multipart data: {parse_error}")
        }
        MultipartError::Payload(payload_error) => {
            format!("Payload error in multipart request: {payload_error}")
        }
        MultipartError::NotConsumed => "Multipart field was not fully consumed.".to_string(),
        MultipartError::Field { name, source } => {
            format!("Error in multipart field '{name}': {source}")
        }
        MultipartError::DuplicateField(name) => {
            format!("Duplicate field '{name}' found in multipart request.")
        }
        MultipartError::MissingField(name) => {
            format!("Required field '{name}' is missing from multipart request.")
        }
        MultipartError::UnknownField(name) => {
            format!("Unknown field '{name}' found in multipart request.")
        }
        _ => err.to_string(),
    };
    ProblemDetails::bad_request().with_detail(detail).into()
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
