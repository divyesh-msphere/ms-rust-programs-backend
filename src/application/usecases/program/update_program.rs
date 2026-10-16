use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::application::services::library_options::{resolve_options, validate_libraries};
use crate::domain::aep::ResourceIdentifier;
use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::{Program, ProgramRepository, UpdateProgram, validate_level};
use crate::domain::program_module::ProgramModule;
use crate::domain::program_setting::{ModuleChange, validate_changes};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

const MAX_NAME_LEN: usize = 120;

pub struct UpdateProgramUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    library_catalog: Arc<dyn LibraryCatalog>,
}

impl UpdateProgramUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, library_catalog: Arc<dyn LibraryCatalog>) -> Self {
        Self { program_repository, library_catalog }
    }
}

#[derive(Debug)]
pub struct UpdateProgramInput {
    pub identifier: ResourceIdentifier,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub level: Option<String>,
    pub active: Option<bool>,
    /// Module changes (add a module, set its mode, replace its settings, switch it off).
    pub modules: Option<Vec<ModuleChange>>,
    pub actor: String,
}

#[derive(Debug)]
pub struct UpdateProgramOutput {
    pub program: Program,
    /// The module manifest after the change, when the request touched modules.
    pub modules: Option<Vec<ProgramModule>>,
}

#[async_trait::async_trait]
impl UseCase<UpdateProgramOutput, UpdateProgramInput> for UpdateProgramUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateProgramInput>) -> Result<UpdateProgramOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in UpdateProgramUseCase")
        })?;

        let name = match input.name {
            Some(name) => {
                let name = name.trim().to_string();
                if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
                    return Err(ApplicationError::unprocessable_entity(&format!(
                        "name must be 1-{MAX_NAME_LEN} characters"
                    )));
                }
                Some(name)
            }
            None => None,
        };
        if let Some(level) = &input.level {
            validate_level(level)?;
        }

        if let Some(changes) = &input.modules {
            validate_changes(changes)?;
            // Libraries are checked BEFORE the transaction opens: no outside call runs inside one.
            validate_libraries(&self.library_catalog, changes).await?;
        }

        let existing = self
            .program_repository
            .get(&input.identifier, false)
            .instrument(info_span!("UpdateProgramUseCase::get"))
            .await?;

        let (program, modules) = self
            .program_repository
            .update(
                existing.id,
                UpdateProgram {
                    name,
                    description: input.description.map(|d| d.map(|d| d.trim().to_string()).filter(|d| !d.is_empty())),
                    level: input.level,
                    active: input.active,
                    modules: input.modules,
                    actor: input.actor,
                },
            )
            .instrument(info_span!("UpdateProgramUseCase::update"))
            .await?;

        let modules = match modules {
            Some(mut modules) => {
                resolve_options(&self.library_catalog, &mut modules).await;
                Some(modules)
            }
            None => None,
        };

        Ok(UpdateProgramOutput { program, modules })
    }
}
