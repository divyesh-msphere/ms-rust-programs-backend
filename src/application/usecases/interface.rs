use crate::application::errors::application_error::ApplicationError;
use async_trait::async_trait;

#[async_trait]
pub trait UseCase<Output, Input> {
    async fn execute(&self, input: Option<Input>) -> Result<Output, ApplicationError>;
}
