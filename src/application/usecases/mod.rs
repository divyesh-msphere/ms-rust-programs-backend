use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

pub mod interface;
pub mod module_definition;

pub struct UseCases {
    pub module_definition: module_definition::ModuleDefinitionUseCases,
}

impl UseCases {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        Self {
            module_definition: module_definition::ModuleDefinitionUseCases::new(
                repositories.module_definition_repository.clone(),
            ),
        }
    }
}
