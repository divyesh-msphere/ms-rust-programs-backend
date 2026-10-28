use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::module_catalog_item::{ModuleCatalogItemNode, ModuleCatalogItemRepository, build_tree};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct GetModuleCatalogTreeUseCase {
    module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>,
}

impl GetModuleCatalogTreeUseCase {
    pub fn new(module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>) -> Self {
        Self { module_catalog_item_repository }
    }
}

#[derive(Debug)]
pub struct GetModuleCatalogTreeInput {
    /// `None` returns the whole tree; a module id returns everything beneath it.
    pub root_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct GetModuleCatalogTreeOutput {
    pub tree: Vec<ModuleCatalogItemNode>,
}

#[async_trait::async_trait]
impl UseCase<GetModuleCatalogTreeOutput, GetModuleCatalogTreeInput> for GetModuleCatalogTreeUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<GetModuleCatalogTreeInput>,
    ) -> Result<GetModuleCatalogTreeOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in GetModuleCatalogTreeUseCase")
        })?;

        let rows = self
            .module_catalog_item_repository
            .list_descendants(input.root_id)
            .instrument(info_span!("GetModuleCatalogTreeUseCase::list_descendants"))
            .await?;

        Ok(GetModuleCatalogTreeOutput {
            tree: build_tree(rows, input.root_id),
        })
    }
}
