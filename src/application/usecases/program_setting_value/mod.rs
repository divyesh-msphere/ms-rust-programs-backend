use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::ProgramRepository;
use crate::domain::program_setting_value::ProgramSettingValueRepository;
use std::sync::Arc;

pub mod batch_update_setting_values;
pub mod list_setting_values;

use batch_update_setting_values::BatchUpdateSettingValuesUseCase;
use list_setting_values::ListSettingValuesUseCase;

pub struct ProgramSettingValueUseCases {
    pub list_setting_values: ListSettingValuesUseCase,
    pub batch_update_setting_values: BatchUpdateSettingValuesUseCase,
}

impl ProgramSettingValueUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        setting_value_repository: Arc<dyn ProgramSettingValueRepository>,
        library_catalog: Arc<dyn LibraryCatalog>,
    ) -> Self {
        Self {
            list_setting_values: ListSettingValuesUseCase::new(program_repository.clone(), setting_value_repository.clone()),
            batch_update_setting_values: BatchUpdateSettingValuesUseCase::new(
                program_repository,
                setting_value_repository,
                library_catalog,
            ),
        }
    }
}
