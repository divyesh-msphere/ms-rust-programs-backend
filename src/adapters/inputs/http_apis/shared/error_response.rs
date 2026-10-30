// src/adapters/inputs/http_apis/shared/error_response.rs

use crate::application::errors::application_error::{ApplicationError, ApplicationErrorKind};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, Error};
use serde::Serialize;
use uuid::Uuid;

/// AEP-193 / RFC 9457 Problem Detail body.
#[derive(Debug, Serialize)]
struct ProblemDetail {
    r#type: &'static str,
    status: u16,
    title: &'static str,
    detail: String,
    instance: String,
    #[serde(flatten)]
    metadata: Option<serde_json::Value>,
}

#[derive(Error, Debug, Display)]
#[display("{detail}")]
pub struct ErrorResponse {
    status: StatusCode,
    r#type: &'static str,
    title: &'static str,
    detail: String,
    metadata: Option<serde_json::Value>,
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        let problem = ProblemDetail {
            r#type: self.r#type,
            status: self.status.as_u16(),
            title: self.title,
            detail: self.detail.clone(),
            instance: Uuid::new_v4().to_string(),
            metadata: self.metadata.clone(),
        };

        HttpResponse::build(self.status)
            .content_type("application/problem+json")
            .json(problem)
    }
}

impl ErrorResponse {
    pub fn map_response_error(error: ApplicationError) -> ErrorResponse {
        let (status, r#type, title) = match error.kind() {
            ApplicationErrorKind::InvalidArgument => {
                (StatusCode::BAD_REQUEST, "INVALID_ARGUMENT", "BadRequest")
            }
            ApplicationErrorKind::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "NotFound"),
            ApplicationErrorKind::AlreadyExists => {
                (StatusCode::CONFLICT, "ALREADY_EXISTS", "Conflict")
            }
            ApplicationErrorKind::PermissionDenied => {
                (StatusCode::FORBIDDEN, "PERMISSION_DENIED", "Forbidden")
            }
            ApplicationErrorKind::Unauthenticated => {
                (StatusCode::UNAUTHORIZED, "UNAUTHENTICATED", "Unauthorized")
            }
            ApplicationErrorKind::Conflict => (StatusCode::CONFLICT, "CONFLICT", "Conflict"),
            ApplicationErrorKind::FailedPrecondition => (
                StatusCode::BAD_REQUEST,
                "FAILED_PRECONDITION",
                "Failed Precondition",
            ),
            ApplicationErrorKind::ResourceExhausted => (
                StatusCode::TOO_MANY_REQUESTS,
                "RESOURCE_EXHAUSTED",
                "Too Many Requests",
            ),
            ApplicationErrorKind::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
                "Payload Too Large",
            ),
            ApplicationErrorKind::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "Unsupported Media Type",
            ),
            ApplicationErrorKind::UnprocessableEntity => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNPROCESSABLE_ENTITY",
                "Unprocessable Entity",
            ),
            ApplicationErrorKind::DeadlineExceeded => (
                StatusCode::GATEWAY_TIMEOUT,
                "DEADLINE_EXCEEDED",
                "Gateway Timeout",
            ),
            ApplicationErrorKind::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                "Internal Server Error",
            ),
        };

        ErrorResponse {
            status,
            r#type,
            title,
            detail: error.message().to_string(),
            metadata: error.metadata().cloned(),
        }
    }
}
