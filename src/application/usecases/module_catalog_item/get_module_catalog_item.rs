use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::module_catalog_item::{ModuleCatalogItem, ModuleCatalogItemRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct GetModuleCatalogItemUseCase {
    module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>,
}

impl GetModuleCatalogItemUseCase {
    pub fn new(module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>) -> Self {
        Self { module_catalog_item_repository }
    }
}

#[derive(Debug)]
pub struct GetModuleCatalogItemInput {
    /// The program (id or slug) whose switches decide what is returned.
    pub program: String,
    pub identifier: ResourceIdentifier,
}

#[derive(Debug)]
pub struct GetModuleCatalogItemOutput {
    /// The item and everything beneath it that the program has switched on.
    pub module_catalog_items: Vec<ModuleCatalogItem>,
}

#[async_trait::async_trait]
impl UseCase<GetModuleCatalogItemOutput, GetModuleCatalogItemInput> for GetModuleCatalogItemUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<GetModuleCatalogItemInput>,
    ) -> Result<GetModuleCatalogItemOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in GetModuleCatalogItemUseCase")
        })?;

        let program = input.program.trim();
        if program.is_empty() {
            return Err(ApplicationError::validation_error("program is required"));
        }

        let module_catalog_items = self
            .module_catalog_item_repository
            .list_enabled_under(program, input.identifier)
            .instrument(info_span!("GetModuleCatalogItemUseCase::list_enabled_under"))
            .await?;

        Ok(GetModuleCatalogItemOutput { module_catalog_items })
    }
}
