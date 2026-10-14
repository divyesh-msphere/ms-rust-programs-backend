use crate::application::errors::application_error::ApplicationError;

/// Sibling of `HttpApiMapper`, same shape: fallible request->input parsing
/// (a malformed id in a gRPC request is a client error, not something to
/// silently coerce to a default value), infallible output->response mapping.
pub trait GrpcApiMapper<UseCaseOutput, UseCaseInput, Response, Request> {
    fn input(request: Request) -> Result<UseCaseInput, ApplicationError>;
    fn output_success(output: UseCaseOutput) -> Response;
}
