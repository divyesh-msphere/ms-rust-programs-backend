use crate::domain::program::ProgramRepository;
use crate::domain::program_setting_value::ProgramSettingValueRepository;
use std::sync::Arc;

pub mod batch_update_module_settings;
pub mod batch_update_setting_values;
pub mod list_setting_values;

use batch_update_module_settings::BatchUpdateModuleSettingsUseCase;
use batch_update_setting_values::BatchUpdateSettingValuesUseCase;
use list_setting_values::ListSettingValuesUseCase;

pub struct ProgramSettingValueUseCases {
    pub list_setting_values: ListSettingValuesUseCase,
    pub batch_update_setting_values: Arc<BatchUpdateSettingValuesUseCase>,
    pub batch_update_module_settings: BatchUpdateModuleSettingsUseCase,
}

impl ProgramSettingValueUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
    ) -> Self {
        let batch = Arc::new(BatchUpdateSettingValuesUseCase::new(
            program_repository.clone(),
            setting_value_repository.clone(),
        ));
        Self {
            list_setting_values: ListSettingValuesUseCase::new(program_repository.clone(), setting_value_repository.clone()),
            batch_update_module_settings: BatchUpdateModuleSettingsUseCase::new(
                program_repository,
                setting_value_repository,
                batch.clone(),
            ),
            batch_update_setting_values: batch,
        }
    }
}
