use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::program_setting_value::{
    ProgramSettingValueRepository, StoredValue, ValueInput, too_many, validate_value,
};
use crate::domain::program_setting::OptionsSource;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const MAX_REPORTED_ERRORS: usize = 10;

pub struct BatchUpdateSettingValuesUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
}

impl BatchUpdateSettingValuesUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
    ) -> Self {
        Self { program_repository, setting_value_repository }
    }
}

#[derive(Debug)]
pub struct BatchUpdateSettingValuesInput {
    pub program: ResourceIdentifier,
    /// The `config_version` the caller read; a stale one is a conflict. `None` skips the check.
    pub version: Option<i32>,
    /// What to save, keyed by setting id. An empty input clears the setting.
    pub values: Vec<(Uuid, ValueInput)>,
    pub actor: String,
}

#[derive(Debug)]
pub struct BatchUpdateSettingValuesOutput {
    pub version: i32,
    /// How many values actually changed.
    pub changed: usize,
    pub values: Vec<StoredValue>,
}

#[async_trait::async_trait]
impl UseCase<BatchUpdateSettingValuesOutput, BatchUpdateSettingValuesInput> for BatchUpdateSettingValuesUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<BatchUpdateSettingValuesInput>,
    ) -> Result<BatchUpdateSettingValuesOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in BatchUpdateSettingValuesUseCase")
        })?;

        if input.values.is_empty() {
            return Err(ApplicationError::validation_error("send at least one value"));
        }
        too_many(input.values.len())?;

        let ids: Vec<Uuid> = input.values.iter().map(|(id, _)| *id).collect();
        let distinct: HashSet<&Uuid> = ids.iter().collect();
        if distinct.len() != ids.len() {
            return Err(ApplicationError::unprocessable_entity("a setting is listed twice"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("BatchUpdateSettingValuesUseCase::get_program"))
            .await?;

        // What each value is checked against: the setting as defined right now.
        let definitions: HashMap<Uuid, _> = self
            .setting_value_repository
            .definitions(program.id, &ids)
            .instrument(info_span!("BatchUpdateSettingValuesUseCase::definitions"))
            .await?
            .into_iter()
            .map(|d| (d.id, d))
            .collect();
        let unknown: Vec<String> = ids.iter().filter(|id| !definitions.contains_key(id)).map(|id| id.to_string()).collect();
        if !unknown.is_empty() {
            return Err(ApplicationError::unprocessable_entity(&format!(
                "not settings of this program: {}",
                unknown.join(", ")
            )));
        }

        let mut changes = Vec::with_capacity(input.values.len());
        let mut errors: Vec<String> = Vec::new();
        for (id, value) in &input.values {
            let definition = &definitions[id];

            let valid_options: Option<HashSet<Uuid>> = match definition.options_source {
                None => None,
                Some(OptionsSource::Manual) => Some(definition.manual_option_ids.clone()),
                // Library options live in global-settings, which this service does not call: the
                // chosen ids are stored as given.
                Some(OptionsSource::Library) => Some(value.option_ids.clone().unwrap_or_default().into_iter().collect()),
            };

            match validate_value(definition, value, valid_options.as_ref()) {
                Ok(change) => changes.push(change),
                Err(message) => errors.push(message),
            }
        }
        if !errors.is_empty() {
            let more = errors.len().saturating_sub(MAX_REPORTED_ERRORS);
            errors.truncate(MAX_REPORTED_ERRORS);
            let suffix = if more > 0 { format!(" (and {more} more)") } else { String::new() };
            return Err(ApplicationError::unprocessable_entity(&format!("{}{suffix}", errors.join("; "))));
        }

        let outcome = self
            .setting_value_repository
            .save(program.id, input.version, changes, &input.actor)
            .instrument(info_span!("BatchUpdateSettingValuesUseCase::save"))
            .await?;

        let (version, values) = self.setting_value_repository.list(program.id, None).await?;
        Ok(BatchUpdateSettingValuesOutput { version: version.max(outcome.version), changed: outcome.changed, values })
    }
}
