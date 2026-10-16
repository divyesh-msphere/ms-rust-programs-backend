use crate::application::errors::application_error::ApplicationError;
use crate::application::services::library_options::active_option_ids;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::library_catalog::{CatalogError, LibraryCatalog};
use crate::domain::program::ProgramRepository;
use crate::domain::program_field_value::{
    ProgramFieldValueRepository, StoredValue, ValueInput, too_many, validate_value,
};
use crate::domain::program_setting::OptionsSource;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const MAX_REPORTED_ERRORS: usize = 10;

pub struct BatchUpdateFieldValuesUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    field_value_repository: Arc<dyn ProgramFieldValueRepository>,
    library_catalog: Arc<dyn LibraryCatalog>,
}

impl BatchUpdateFieldValuesUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        field_value_repository: Arc<dyn ProgramFieldValueRepository>,
        library_catalog: Arc<dyn LibraryCatalog>,
    ) -> Self {
        Self { program_repository, field_value_repository, library_catalog }
    }
}

#[derive(Debug)]
pub struct BatchUpdateFieldValuesInput {
    pub program: ResourceIdentifier,
    /// The `config_version` the caller read; a stale one is a conflict. `None` skips the check.
    pub version: Option<i32>,
    /// What to save, keyed by setting id. An empty input clears the setting.
    pub values: Vec<(Uuid, ValueInput)>,
    pub actor: String,
}

#[derive(Debug)]
pub struct BatchUpdateFieldValuesOutput {
    pub version: i32,
    /// How many values actually changed.
    pub changed: usize,
    pub values: Vec<StoredValue>,
}

#[async_trait::async_trait]
impl UseCase<BatchUpdateFieldValuesOutput, BatchUpdateFieldValuesInput> for BatchUpdateFieldValuesUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<BatchUpdateFieldValuesInput>,
    ) -> Result<BatchUpdateFieldValuesOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in BatchUpdateFieldValuesUseCase")
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
            .instrument(info_span!("BatchUpdateFieldValuesUseCase::get_program"))
            .await?;

        // What each value is checked against: the setting as defined right now.
        let definitions: HashMap<Uuid, _> = self
            .field_value_repository
            .definitions(program.id, &ids)
            .instrument(info_span!("BatchUpdateFieldValuesUseCase::definitions"))
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

        // The options a library-backed choice may pick from, read through the cache BEFORE the
        // transaction opens (no outside call ever runs inside one).
        let library_ids: Vec<Uuid> = definitions
            .values()
            .filter(|d| d.options_source == Some(OptionsSource::Library))
            .filter_map(|d| d.library_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let libraries = if library_ids.is_empty() || !self.library_catalog.enabled() {
            HashMap::new()
        } else {
            active_option_ids(&self.library_catalog, library_ids).await
        };

        let mut changes = Vec::with_capacity(input.values.len());
        let mut errors: Vec<String> = Vec::new();
        for (id, value) in &input.values {
            let definition = &definitions[id];

            let valid_options: Option<HashSet<Uuid>> = match definition.options_source {
                None => None,
                Some(OptionsSource::Manual) => Some(definition.manual_option_ids.clone()),
                Some(OptionsSource::Library) => match definition.library_id.and_then(|lib| libraries.get(&lib)) {
                    Some(Ok(active)) => Some(active.clone()),
                    Some(Err(CatalogError::NotFound)) => {
                        errors.push(format!("{:?}: its library no longer exists in global-settings", definition.name));
                        continue;
                    }
                    Some(Err(CatalogError::Unavailable(reason))) => {
                        return Err(ApplicationError::failed_precondition(&format!(
                            "global-settings is unavailable, so {:?} cannot be checked: {reason}",
                            definition.name
                        )));
                    }
                    // Integration switched off: the options cannot be checked, so they are taken as given.
                    None => Some(value.option_ids.clone().unwrap_or_default().into_iter().collect()),
                },
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
            .field_value_repository
            .save(program.id, input.version, changes, &input.actor)
            .instrument(info_span!("BatchUpdateFieldValuesUseCase::save"))
            .await?;

        let (version, values) = self.field_value_repository.list(program.id, None).await?;
        Ok(BatchUpdateFieldValuesOutput { version: version.max(outcome.version), changed: outcome.changed, values })
    }
}
