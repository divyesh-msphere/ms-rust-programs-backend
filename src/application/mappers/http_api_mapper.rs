use crate::application::errors::application_error::ApplicationError;

pub trait HttpApiMapper<UseCaseOutput, UseCaseInput, Presenter, Payload> {
    /// Fallible: the payload's ids are strings, and one that is not a UUID is a
    /// malformed request rather than a value the domain should ever be handed.
    fn input(payload: Payload) -> Result<UseCaseInput, ApplicationError>;
    fn output_success(output: UseCaseOutput) -> Presenter;
}
