use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::ProgramRepository;
use crate::domain::program_module::ProgramModuleRepository;
use std::sync::Arc;

pub mod list_program_modules;

use list_program_modules::ListProgramModulesUseCase;

pub struct ProgramModuleUseCases {
    pub list_program_modules: ListProgramModulesUseCase,
}

impl ProgramModuleUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        program_module_repository: Arc<dyn ProgramModuleRepository>,
        library_catalog: Arc<dyn LibraryCatalog>,
    ) -> Self {
        Self {
            list_program_modules: ListProgramModulesUseCase::new(program_repository, program_module_repository, library_catalog),
        }
    }
}
