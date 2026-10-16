use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::program_field_value::{ProgramFieldValueRepository, StoredValue};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct ListFieldValuesUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    field_value_repository: Arc<dyn ProgramFieldValueRepository>,
}

impl ListFieldValuesUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        field_value_repository: Arc<dyn ProgramFieldValueRepository>,
    ) -> Self {
        Self { program_repository, field_value_repository }
    }
}

#[derive(Debug)]
pub struct ListFieldValuesInput {
    pub program: ResourceIdentifier,
    /// Only one module's values (its key).
    pub module_key: Option<String>,
}

#[derive(Debug)]
pub struct ListFieldValuesOutput {
    /// The program's `config_version`: send it back with a save to detect a concurrent change.
    pub version: i32,
    pub values: Vec<StoredValue>,
}

#[async_trait::async_trait]
impl UseCase<ListFieldValuesOutput, ListFieldValuesInput> for ListFieldValuesUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListFieldValuesInput>) -> Result<ListFieldValuesOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListFieldValuesUseCase")
        })?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListFieldValuesUseCase::get_program"))
            .await?;

        let (version, values) = self
            .field_value_repository
            .list(program.id, input.module_key.as_deref())
            .instrument(info_span!("ListFieldValuesUseCase::list"))
            .await?;

        Ok(ListFieldValuesOutput { version, values })
    }
}
