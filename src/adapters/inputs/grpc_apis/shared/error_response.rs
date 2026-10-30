use crate::application::errors::application_error::{ApplicationError, ApplicationErrorKind};
use connectrpc::{ConnectError, ErrorCode};

/// gRPC counterpart of `http_apis::shared::error_response::ErrorResponse`:
/// same `ApplicationErrorKind` match, different output type. `metadata()`
/// (the HTTP side's RFC 9457 problem-detail extra JSON) has no first-class
/// gRPC equivalent and is dropped here rather than shoehorned into a
/// `ConnectError` detail.
pub struct GrpcErrorResponse;

impl GrpcErrorResponse {
    pub fn map_response_error(error: ApplicationError) -> ConnectError {
        let code = match error.kind() {
            ApplicationErrorKind::InvalidArgument => ErrorCode::InvalidArgument,
            ApplicationErrorKind::NotFound => ErrorCode::NotFound,
            ApplicationErrorKind::AlreadyExists => ErrorCode::AlreadyExists,
            ApplicationErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            ApplicationErrorKind::Unauthenticated => ErrorCode::Unauthenticated,
            ApplicationErrorKind::FailedPrecondition => ErrorCode::FailedPrecondition,
            ApplicationErrorKind::Conflict => ErrorCode::Aborted,
            ApplicationErrorKind::ResourceExhausted => ErrorCode::ResourceExhausted,
            ApplicationErrorKind::PayloadTooLarge => ErrorCode::ResourceExhausted,
            ApplicationErrorKind::UnsupportedMediaType => ErrorCode::InvalidArgument,
            // gRPC's status space has no HTTP-422 equivalent; this kind means
            // "well-formed request, malformed resource-format value" (e.g.
            // AEP-122 `code` pattern) — still the client's fault, so it
            // collapses to the code that already means that.
            ApplicationErrorKind::UnprocessableEntity => ErrorCode::InvalidArgument,
            ApplicationErrorKind::DeadlineExceeded => ErrorCode::DeadlineExceeded,
            ApplicationErrorKind::Internal => ErrorCode::Internal,
        };
        ConnectError::new(code, error.message())
    }
}
