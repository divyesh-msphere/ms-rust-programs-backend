use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::module_definition::{ModuleDefinition, ModuleDefinitionRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListModuleDefinitionsUseCase {
    module_definition_repository: Arc<dyn ModuleDefinitionRepository>,
}

impl ListModuleDefinitionsUseCase {
    pub fn new(module_definition_repository: Arc<dyn ModuleDefinitionRepository>) -> Self {
        Self { module_definition_repository }
    }
}

#[derive(Debug)]
pub struct ListModuleDefinitionsInput {
    pub parent_id: Option<Uuid>,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListModuleDefinitionsOutput {
    pub module_definitions: Vec<ModuleDefinition>,
    pub next_page_token: Option<Uuid>,
    pub total_size: i64,
}

#[async_trait::async_trait]
impl UseCase<ListModuleDefinitionsOutput, ListModuleDefinitionsInput> for ListModuleDefinitionsUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<ListModuleDefinitionsInput>,
    ) -> Result<ListModuleDefinitionsOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListModuleDefinitionsUseCase")
        })?;

        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let (module_definitions, next_page_token, total_size) = self
            .module_definition_repository
            .list(input.parent_id, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListModuleDefinitionsUseCase::list"))
            .await?;

        Ok(ListModuleDefinitionsOutput {
            module_definitions,
            next_page_token,
            total_size,
        })
    }
}
