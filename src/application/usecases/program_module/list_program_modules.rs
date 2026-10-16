use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::application::services::library_options::resolve_options;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::ProgramRepository;
use crate::domain::program_module::{ProgramModule, ProgramModuleRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct ListProgramModulesUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    program_module_repository: Arc<dyn ProgramModuleRepository>,
    library_catalog: Arc<dyn LibraryCatalog>,
}

impl ListProgramModulesUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        program_module_repository: Arc<dyn ProgramModuleRepository>,
        library_catalog: Arc<dyn LibraryCatalog>,
    ) -> Self {
        Self { program_repository, program_module_repository, library_catalog }
    }
}

#[derive(Debug)]
pub struct ListProgramModulesInput {
    pub program: ResourceIdentifier,
    /// Hide modules the program has switched off (what the frontend draws).
    pub enabled_only: bool,
    /// Fill library-backed settings with their library's options (from the catalog's cache).
    pub include_options: bool,
}

#[derive(Debug)]
pub struct ListProgramModulesOutput {
    pub modules: Vec<ProgramModule>,
}

#[async_trait::async_trait]
impl UseCase<ListProgramModulesOutput, ListProgramModulesInput> for ListProgramModulesUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<ListProgramModulesInput>,
    ) -> Result<ListProgramModulesOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListProgramModulesUseCase")
        })?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListProgramModulesUseCase::get_program"))
            .await?;

        let mut modules = self
            .program_module_repository
            .list(program.id, input.enabled_only)
            .instrument(info_span!("ListProgramModulesUseCase::list"))
            .await?;

        if input.include_options {
            resolve_options(&self.library_catalog, &mut modules).await;
        }

        Ok(ListProgramModulesOutput { modules })
    }
}
