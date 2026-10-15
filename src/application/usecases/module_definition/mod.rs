use crate::domain::module_definition::ModuleDefinitionRepository;
use std::sync::Arc;

pub mod get_module_definition;
pub mod get_module_definition_tree;
pub mod list_module_definitions;

use get_module_definition::GetModuleDefinitionUseCase;
use get_module_definition_tree::GetModuleDefinitionTreeUseCase;
use list_module_definitions::ListModuleDefinitionsUseCase;

pub struct ModuleDefinitionUseCases {
    pub list_module_definitions: ListModuleDefinitionsUseCase,
    pub get_module_definition: GetModuleDefinitionUseCase,
    pub get_module_definition_tree: GetModuleDefinitionTreeUseCase,
}

impl ModuleDefinitionUseCases {
    pub fn new(module_definition_repository: Arc<dyn ModuleDefinitionRepository>) -> Self {
        Self {
            list_module_definitions: ListModuleDefinitionsUseCase::new(module_definition_repository.clone()),
            get_module_definition: GetModuleDefinitionUseCase::new(module_definition_repository.clone()),
            get_module_definition_tree: GetModuleDefinitionTreeUseCase::new(module_definition_repository),
        }
    }
}
