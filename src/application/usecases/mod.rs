use crate::domain::library_catalog::LibraryCatalog;
use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

pub mod interface;
pub mod module_definition;
pub mod program;
pub mod program_field_value;
pub mod program_module;

pub struct UseCases {
    pub module_definition: module_definition::ModuleDefinitionUseCases,
    pub program: program::ProgramUseCases,
    pub program_module: program_module::ProgramModuleUseCases,
    pub program_field_value: program_field_value::ProgramFieldValueUseCases,
}

impl UseCases {
    pub fn new(repositories: Arc<Repositories>, library_catalog: Arc<dyn LibraryCatalog>) -> Self {
        Self {
            module_definition: module_definition::ModuleDefinitionUseCases::new(
                repositories.module_definition_repository.clone(),
            ),
            program: program::ProgramUseCases::new(repositories.program_repository.clone(), library_catalog.clone()),
            program_module: program_module::ProgramModuleUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_module_repository.clone(),
                library_catalog.clone(),
            ),
            program_field_value: program_field_value::ProgramFieldValueUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_field_value_repository.clone(),
                library_catalog,
            ),
        }
    }
}
