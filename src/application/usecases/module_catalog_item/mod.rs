use crate::domain::module_catalog_item::ModuleCatalogItemRepository;
use std::sync::Arc;

pub mod get_module_catalog_item;
pub mod get_module_catalog_tree;
pub mod list_module_catalog;

use get_module_catalog_item::GetModuleCatalogItemUseCase;
use get_module_catalog_tree::GetModuleCatalogTreeUseCase;
use list_module_catalog::ListModuleCatalogUseCase;

pub struct ModuleCatalogItemUseCases {
    pub list_module_catalog: ListModuleCatalogUseCase,
    pub get_module_catalog_item: GetModuleCatalogItemUseCase,
    pub get_module_catalog_tree: GetModuleCatalogTreeUseCase,
}

impl ModuleCatalogItemUseCases {
    pub fn new(module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>) -> Self {
        Self {
            list_module_catalog: ListModuleCatalogUseCase::new(module_catalog_item_repository.clone()),
            get_module_catalog_item: GetModuleCatalogItemUseCase::new(module_catalog_item_repository.clone()),
            get_module_catalog_tree: GetModuleCatalogTreeUseCase::new(module_catalog_item_repository),
        }
    }
}
