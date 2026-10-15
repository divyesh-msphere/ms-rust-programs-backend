use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::module_definition::{ModuleDefinition, ModuleDefinitionRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct GetModuleDefinitionUseCase {
    module_definition_repository: Arc<dyn ModuleDefinitionRepository>,
}

impl GetModuleDefinitionUseCase {
    pub fn new(module_definition_repository: Arc<dyn ModuleDefinitionRepository>) -> Self {
        Self { module_definition_repository }
    }
}

#[derive(Debug)]
pub struct GetModuleDefinitionInput {
    pub identifier: ResourceIdentifier,
}

#[derive(Debug)]
pub struct GetModuleDefinitionOutput {
    pub module_definition: ModuleDefinition,
}

#[async_trait::async_trait]
impl UseCase<GetModuleDefinitionOutput, GetModuleDefinitionInput> for GetModuleDefinitionUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<GetModuleDefinitionInput>,
    ) -> Result<GetModuleDefinitionOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in GetModuleDefinitionUseCase")
        })?;

        let module_definition = self
            .module_definition_repository
            .get(input.identifier)
            .instrument(info_span!("GetModuleDefinitionUseCase::get"))
            .await?;

        Ok(GetModuleDefinitionOutput { module_definition })
    }
}
