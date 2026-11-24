use super::batch_update_setting_values::{
    BatchUpdateSettingValuesInput, BatchUpdateSettingValuesOutput, BatchUpdateSettingValuesUseCase,
};
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::program_setting_value::{ProgramSettingValueRepository, ValueInput, too_many};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const MAX_REPORTED_ERRORS: usize = 10;

/// Saves settings of several modules in one request. A setting is named by its id or by its key
/// inside the module; the save itself is the ordinary batch save (one transaction).
pub struct BatchUpdateModuleSettingsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
    batch: Arc<BatchUpdateSettingValuesUseCase>,
}

impl BatchUpdateModuleSettingsUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
        batch: Arc<BatchUpdateSettingValuesUseCase>,
    ) -> Self {
        Self { program_repository, setting_value_repository, batch }
    }
}

#[derive(Debug)]
pub struct ModuleSettingItem {
    /// The setting's id, or...
    pub id: Option<Uuid>,
    /// ...its key within the module. Exactly one of the two.
    pub key: Option<String>,
    pub value: ValueInput,
}

#[derive(Debug)]
pub struct ModuleSettingsInput {
    pub key: String,
    pub settings: Vec<ModuleSettingItem>,
}

#[derive(Debug)]
pub struct BatchUpdateModuleSettingsInput {
    pub program: ResourceIdentifier,
    pub version: Option<i32>,
    pub modules: Vec<ModuleSettingsInput>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<BatchUpdateSettingValuesOutput, BatchUpdateModuleSettingsInput> for BatchUpdateModuleSettingsUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<BatchUpdateModuleSettingsInput>,
    ) -> Result<BatchUpdateSettingValuesOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in BatchUpdateModuleSettingsUseCase")
        })?;
        if input.modules.is_empty() {
            return Err(ApplicationError::validation_error("send at least one module"));
        }
        let total: usize = input.modules.iter().map(|m| m.settings.len()).sum();
        if total == 0 {
            return Err(ApplicationError::validation_error("send at least one setting"));
        }
        too_many(total)?;

        let mut seen_modules = HashSet::new();
        for module in &input.modules {
            if !seen_modules.insert(module.key.as_str()) {
                return Err(ApplicationError::unprocessable_entity(&format!("module {:?} is listed twice", module.key)));
            }
        }

        let program = self.program_repository.get(&input.program, false).await?;

        // Settings named by key -> ids.
        let mut key_refs: Vec<(String, String)> = Vec::new();
        for module in &input.modules {
            for item in &module.settings {
                match (&item.id, &item.key) {
                    (Some(_), Some(_)) | (None, None) => {
                        return Err(ApplicationError::unprocessable_entity(&format!(
                            "module {:?}: each setting needs exactly one of `id` or `key`",
                            module.key
                        )));
                    }
                    (None, Some(key)) => key_refs.push((module.key.clone(), key.clone())),
                    _ => {}
                }
            }
        }
        let by_key = if key_refs.is_empty() {
            Default::default()
        } else {
            self.setting_value_repository.setting_ids_by_key(program.id, &key_refs).await?
        };

        let mut errors: Vec<String> = Vec::new();
        let mut resolved: Vec<(String, Uuid, ValueInput)> = Vec::with_capacity(total);
        for module in input.modules {
            for item in module.settings {
                let id = match (item.id, item.key) {
                    (Some(id), _) => id,
                    (None, Some(key)) => match by_key.get(&(module.key.clone(), key.clone())) {
                        Some(id) => *id,
                        None => {
                            errors.push(format!("module {:?} has no setting {key:?}", module.key));
                            continue;
                        }
                    },
                    _ => continue,
                };
                resolved.push((module.key.clone(), id, item.value));
            }
        }

        // Every id must be a live setting of its own module.
        let ids: Vec<Uuid> = resolved.iter().map(|(_, id, _)| *id).collect();
        let definitions = self.setting_value_repository.definitions(program.id, &ids).await?;
        for (module_key, id, _) in &resolved {
            match definitions.iter().find(|d| d.id == *id) {
                None => errors.push(format!("module {module_key:?}: setting {id} is not a setting of this program")),
                Some(d) if d.module_key != *module_key => errors.push(format!(
                    "setting {:?} ({id}) belongs to module {:?}, not {module_key:?}",
                    d.name, d.module_key
                )),
                _ => {}
            }
        }
        if !errors.is_empty() {
            let more = errors.len().saturating_sub(MAX_REPORTED_ERRORS);
            errors.truncate(MAX_REPORTED_ERRORS);
            let suffix = if more > 0 { format!(" (and {more} more)") } else { String::new() };
            return Err(ApplicationError::unprocessable_entity(&format!("{}{suffix}", errors.join("; "))));
        }

        self.batch
            .execute(Some(BatchUpdateSettingValuesInput {
                program: input.program,
                version: input.version,
                values: resolved.into_iter().map(|(_, id, value)| (id, value)).collect(),
                actor: input.actor,
            }))
            .await
    }
}
