use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::module_definition::{ModuleDefinitionNode, ModuleDefinitionRepository, build_tree};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct GetModuleDefinitionTreeUseCase {
    module_definition_repository: Arc<dyn ModuleDefinitionRepository>,
}

impl GetModuleDefinitionTreeUseCase {
    pub fn new(module_definition_repository: Arc<dyn ModuleDefinitionRepository>) -> Self {
        Self { module_definition_repository }
    }
}

#[derive(Debug)]
pub struct GetModuleDefinitionTreeInput {
    /// `None` returns the whole tree; a module id returns everything beneath it.
    pub root_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct GetModuleDefinitionTreeOutput {
    pub tree: Vec<ModuleDefinitionNode>,
}

#[async_trait::async_trait]
impl UseCase<GetModuleDefinitionTreeOutput, GetModuleDefinitionTreeInput> for GetModuleDefinitionTreeUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<GetModuleDefinitionTreeInput>,
    ) -> Result<GetModuleDefinitionTreeOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in GetModuleDefinitionTreeUseCase")
        })?;

        let rows = self
            .module_definition_repository
            .list_descendants(input.root_id)
            .instrument(info_span!("GetModuleDefinitionTreeUseCase::list_descendants"))
            .await?;

        Ok(GetModuleDefinitionTreeOutput {
            tree: build_tree(rows, input.root_id),
        })
    }
}
