use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::program_setting_value::{ProgramSettingValueRepository, StoredValue};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct ListSettingValuesUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
}

impl ListSettingValuesUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
    ) -> Self {
        Self { program_repository, setting_value_repository }
    }
}

#[derive(Debug)]
pub struct ListSettingValuesInput {
    pub program: ResourceIdentifier,
    /// Only one module's values (its key).
    pub module_key: Option<String>,
}

#[derive(Debug)]
pub struct ListSettingValuesOutput {
    /// The program's `config_version`: send it back with a save to detect a concurrent change.
    pub version: i32,
    pub values: Vec<StoredValue>,
}

#[async_trait::async_trait]
impl UseCase<ListSettingValuesOutput, ListSettingValuesInput> for ListSettingValuesUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListSettingValuesInput>) -> Result<ListSettingValuesOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListSettingValuesUseCase")
        })?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListSettingValuesUseCase::get_program"))
            .await?;

        let (version, values) = self
            .setting_value_repository
            .list(program.id, input.module_key.as_deref())
            .instrument(info_span!("ListSettingValuesUseCase::list"))
            .await?;

        Ok(ListSettingValuesOutput { version, values })
    }
}
