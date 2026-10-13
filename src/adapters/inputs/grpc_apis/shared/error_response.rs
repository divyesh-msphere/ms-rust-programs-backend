use crate::application::errors::application_error::{ApplicationError, ApplicationErrorKind};
use ms_telemetry_rs::tracing::error;
use tonic::{Code, Status};

pub struct GrpcErrorResponse;

impl GrpcErrorResponse {
    pub fn map_response_error(error: ApplicationError) -> Status {
        error!("gRPC Error: {:?}", error);

        let code = match error.kind() {
            ApplicationErrorKind::InvalidArgument => Code::InvalidArgument,
            ApplicationErrorKind::NotFound => Code::NotFound,
            ApplicationErrorKind::AlreadyExists => Code::AlreadyExists,
            ApplicationErrorKind::PermissionDenied => Code::PermissionDenied,
            ApplicationErrorKind::Unauthenticated => Code::Unauthenticated,
            ApplicationErrorKind::FailedPrecondition => Code::FailedPrecondition,
            ApplicationErrorKind::ResourceExhausted => Code::ResourceExhausted,
            ApplicationErrorKind::Internal => Code::Internal,
        };

        Status::new(code, error.message())
    }
}
