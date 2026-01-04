use actix_multipart::MultipartError;
use actix_web::{
    HttpRequest, HttpResponse, ResponseError,
    error::{JsonPayloadError, QueryPayloadError},
    http::StatusCode,
};
use domain::errors::TermsOfUseError;
use tracing::error;

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

#[tracing::instrument]
pub fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    error!(error = ?err, "json payload error");

    let safe_detail = match &err {
        JsonPayloadError::Deserialize(_) => "Invalid JSON",
        JsonPayloadError::ContentType => "Content type must be application/json",
        JsonPayloadError::Overflow { .. } => "Payload size exceeded",
        JsonPayloadError::Payload(_) => "Payload error",
        _ => "Bad request",
    };

    ProblemDetails::bad_request()
        .with_detail(safe_detail.to_string())
        .into()
}

#[tracing::instrument]
pub fn query_error_handler(err: QueryPayloadError, _req: &HttpRequest) -> actix_web::Error {
    error!(error = ?err, "query payload error");

    let safe_detail = match &err {
        QueryPayloadError::Deserialize(_) => "Invalid query parameter",
        _ => "Bad request",
    };

    ProblemDetails::bad_request()
        .with_detail(safe_detail.to_string())
        .into()
}

#[tracing::instrument]
pub fn multipart_error_handler(err: MultipartError, _req: &HttpRequest) -> actix_web::Error {
    let safe_detail = match err {
        MultipartError::ContentTypeMissing => "Content-Type header is missing. Multipart required.",
        MultipartError::ContentTypeParse => "Failed to parse Content-Type header.",
        MultipartError::ContentTypeIncompatible => "Content-Type not compatible with multipart",
        MultipartError::BoundaryMissing => "Boundary parameter missing from Content-Type",
        MultipartError::ContentDispositionMissing => {
            "Content-Disposition header missing in multipart"
        }
        MultipartError::ContentDispositionNameMissing => {
            "Name parameter missing in Content-Disposition"
        }
        MultipartError::Nested => "Nested multipart not supported",
        MultipartError::Incomplete => "Multipart stream ended unexpectedly",
        MultipartError::Parse(_) => "Failed to parse multipart data",
        MultipartError::Payload(_) => "Payload error in multipart request",
        MultipartError::NotConsumed => "Multipart field was not fully consumed",
        MultipartError::Field { .. } => "Error in multipart field",
        MultipartError::DuplicateField(_) => "Duplicate field in multipart request",
        MultipartError::MissingField(_) => "Required multipart field missing",
        MultipartError::UnknownField(_) => "Unknown field in multipart request",
        _ => "Bad multipart request",
    };

    error!(error = ?err, "multipart error");

    ProblemDetails::bad_request()
        .with_detail(safe_detail.to_string())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    // TermsOfUseError mapping tests
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

    // ResponseError trait tests
    #[test]
    fn test_problem_details_response_error() {
        let problem = ProblemDetails::bad_request().with_detail("Test error");
        let response = problem.error_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_problem_details_status_code() {
        let problem = ProblemDetails::not_found();
        assert_eq!(problem.status_code(), StatusCode::NOT_FOUND);
    }

    // JSON error handler tests
    #[test]
    fn test_json_error_handler_deserialize() {
        let req = TestRequest::default().to_http_request();
        let error = JsonPayloadError::Deserialize(
            serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err(),
        );

        let result = json_error_handler(error, &req);

        assert!(result.to_string().contains("400") || result.to_string().contains("Bad Request"));
    }

    #[test]
    fn test_json_error_handler_content_type() {
        let req = TestRequest::default().to_http_request();
        let error = JsonPayloadError::ContentType;

        let result = json_error_handler(error, &req);

        assert!(
            result
                .to_string()
                .contains("Content type must be application/json")
        );
    }

    #[test]
    fn test_json_error_handler_overflow() {
        let req = TestRequest::default().to_http_request();
        let error = JsonPayloadError::Overflow { limit: 1024 };

        let result = json_error_handler(error, &req);
        assert!(result.to_string().contains("Payload size exceeded"));
    }

    #[test]
    fn test_json_error_handler_payload() {
        let req = TestRequest::default().to_http_request();
        let error = JsonPayloadError::Payload(actix_web::error::PayloadError::Incomplete(None));

        let result = json_error_handler(error, &req);
        assert!(result.to_string().contains("Payload error"));
    }

    // Multipart error handler tests
    #[test]
    fn test_multipart_error_handler_content_type_missing() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::ContentTypeMissing;

        let result = multipart_error_handler(error, &req);

        assert!(
            result
                .to_string()
                .contains("Content-Type header is missing")
        );
    }

    #[test]
    fn test_multipart_error_handler_content_type_parse() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::ContentTypeParse;

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Failed to parse Content-Type header")
        );
    }

    #[test]
    fn test_multipart_error_handler_content_type_incompatible() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::ContentTypeIncompatible;

        let result = multipart_error_handler(error, &req);
        assert!(result.to_string().contains("not compatible with multipart"));
    }

    #[test]
    fn test_multipart_error_handler_boundary_missing() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::BoundaryMissing;

        let result = multipart_error_handler(error, &req);
        assert!(result.to_string().contains("Boundary parameter missing"));
    }

    #[test]
    fn test_multipart_error_handler_content_disposition_missing() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::ContentDispositionMissing;

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Content-Disposition header missing")
        );
    }

    #[test]
    fn test_multipart_error_handler_content_disposition_name_missing() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::ContentDispositionNameMissing;

        let result = multipart_error_handler(error, &req);
        assert!(result.to_string().contains("Name parameter missing"));
    }

    #[test]
    fn test_multipart_error_handler_nested() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::Nested;

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Nested multipart not supported")
        );
    }

    #[test]
    fn test_multipart_error_handler_incomplete() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::Incomplete;

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Multipart stream ended unexpectedly")
        );
    }

    #[test]
    fn test_multipart_error_handler_not_consumed() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::NotConsumed;

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Multipart field was not fully consumed")
        );
    }

    #[test]
    fn test_multipart_error_handler_duplicate_field() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::DuplicateField("file".to_string());

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Duplicate field in multipart request")
        );
    }

    #[test]
    fn test_multipart_error_handler_missing_field() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::MissingField("file".to_string());

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Required multipart field missing")
        );
    }

    #[test]
    fn test_multipart_error_handler_unknown_field() {
        let req = TestRequest::default().to_http_request();
        let error = MultipartError::UnknownField("unknown".to_string());

        let result = multipart_error_handler(error, &req);
        assert!(
            result
                .to_string()
                .contains("Unknown field in multipart request")
        );
    }
}
