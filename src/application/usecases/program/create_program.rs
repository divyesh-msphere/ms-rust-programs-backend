use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::application::services::library_options::{resolve_options, validate_libraries};
use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::{
    DEFAULT_PROGRAM_LEVEL, NewProgram, Program, ProgramRepository, validate_level, validate_slug,
};
use crate::domain::program_module::ProgramModule;
use crate::domain::program_setting::{ModuleChange, validate_changes};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const MAX_NAME_LEN: usize = 120;

pub struct CreateProgramUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    library_catalog: Arc<dyn LibraryCatalog>,
}

impl CreateProgramUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, library_catalog: Arc<dyn LibraryCatalog>) -> Self {
        Self { program_repository, library_catalog }
    }
}

#[derive(Debug)]
pub struct CreateProgramInput {
    pub id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub level: Option<String>,
    /// The modules the admin picked, each optionally with its mode and settings.
    pub modules: Vec<ModuleChange>,
    pub actor: String,
}

#[derive(Debug)]
pub struct CreateProgramOutput {
    pub program: Program,
    pub modules: Vec<ProgramModule>,
}

#[async_trait::async_trait]
impl UseCase<CreateProgramOutput, CreateProgramInput> for CreateProgramUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateProgramInput>) -> Result<CreateProgramOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in CreateProgramUseCase")
        })?;

        let name = input.name.trim().to_string();
        if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
            return Err(ApplicationError::unprocessable_entity(&format!(
                "name must be 1-{MAX_NAME_LEN} characters"
            )));
        }
        validate_slug(&input.slug)?;
        let level = input.level.unwrap_or_else(|| DEFAULT_PROGRAM_LEVEL.to_string());
        validate_level(&level)?;
        // A module named twice with nothing else on it is harmless; anything else is checked.
        let mut modules: Vec<ModuleChange> = Vec::with_capacity(input.modules.len());
        for change in input.modules {
            if !change.has_changes() && modules.iter().any(|m| m.key == change.key && !m.has_changes()) {
                continue;
            }
            modules.push(change);
        }
        validate_changes(&modules)?;
        // Libraries are checked BEFORE the transaction opens: no outside call runs inside one.
        validate_libraries(&self.library_catalog, &modules).await?;

        let (program, modules) = self
            .program_repository
            .create(NewProgram {
                id: input.id,
                name,
                slug: input.slug,
                description: input.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
                level,
                modules,
                actor: input.actor,
            })
            .instrument(info_span!("CreateProgramUseCase::create"))
            .await?;

        let mut modules = modules;
        resolve_options(&self.library_catalog, &mut modules).await;

        Ok(CreateProgramOutput { program, modules })
    }
}
