use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::ProgramRepository;
use crate::domain::program_field_value::ProgramFieldValueRepository;
use std::sync::Arc;

pub mod batch_update_field_values;
pub mod list_field_values;

use batch_update_field_values::BatchUpdateFieldValuesUseCase;
use list_field_values::ListFieldValuesUseCase;

pub struct ProgramFieldValueUseCases {
    pub list_field_values: ListFieldValuesUseCase,
    pub batch_update_field_values: BatchUpdateFieldValuesUseCase,
}

impl ProgramFieldValueUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        field_value_repository: Arc<dyn ProgramFieldValueRepository>,
        library_catalog: Arc<dyn LibraryCatalog>,
    ) -> Self {
        Self {
            list_field_values: ListFieldValuesUseCase::new(program_repository.clone(), field_value_repository.clone()),
            batch_update_field_values: BatchUpdateFieldValuesUseCase::new(
                program_repository,
                field_value_repository,
                library_catalog,
            ),
        }
    }
}
