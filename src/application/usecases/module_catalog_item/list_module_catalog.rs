use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::module_catalog_item::{ModuleCatalogItem, ModuleCatalogItemRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListModuleCatalogUseCase {
    module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>,
}

impl ListModuleCatalogUseCase {
    pub fn new(module_catalog_item_repository: Arc<dyn ModuleCatalogItemRepository>) -> Self {
        Self { module_catalog_item_repository }
    }
}

#[derive(Debug)]
pub struct ListModuleCatalogInput {
    pub parent_id: Option<Uuid>,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListModuleCatalogOutput {
    pub module_catalog: Vec<ModuleCatalogItem>,
    pub next_page_token: Option<Uuid>,
    pub total_size: i64,
}

#[async_trait::async_trait]
impl UseCase<ListModuleCatalogOutput, ListModuleCatalogInput> for ListModuleCatalogUseCase {
    #[instrument(skip(self))]
    async fn execute(
        &self,
        input: Option<ListModuleCatalogInput>,
    ) -> Result<ListModuleCatalogOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListModuleCatalogUseCase")
        })?;

        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let (module_catalog, next_page_token, total_size) = self
            .module_catalog_item_repository
            .list(input.parent_id, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListModuleCatalogUseCase::list"))
            .await?;

        Ok(ListModuleCatalogOutput {
            module_catalog,
            next_page_token,
            total_size,
        })
    }
}
