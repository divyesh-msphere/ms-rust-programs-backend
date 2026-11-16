use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::module_catalog_item::{ModuleCatalogItem, ModuleCatalogItemRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;
const MAX_KEYS: usize = 200;

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
    /// Text to find in a module's name, at any depth. Blank means no search.
    pub search: Option<String>,
    /// Module keys or ids: return exactly these modules (and nothing beneath them).
    pub keys: Vec<String>,
    /// With `checked`: the program (id or slug) whose ticks decide what is returned.
    pub program: Option<String>,
    /// `true`: only the modules `program` has ticked; `false`: the ones it has not. Needs `program`;
    /// `program` alone means `true`.
    pub checked: Option<bool>,
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

        if input.keys.len() > MAX_KEYS {
            return Err(ApplicationError::validation_error(&format!("keys takes at most {MAX_KEYS} modules")));
        }
        let program = input.program.as_deref().map(str::trim).filter(|text| !text.is_empty());
        if input.checked.is_some() && program.is_none() {
            return Err(ApplicationError::validation_error("checked needs a program (its id or slug)"));
        }
        let checked = program.map(|program| (program.to_string(), input.checked.unwrap_or(true)));
        let search = input.search.as_deref().map(str::trim).filter(|text| !text.is_empty()).map(str::to_string);

        let (module_catalog, next_page_token, total_size) = self
            .module_catalog_item_repository
            .list(input.parent_id, search.as_deref(), &input.keys, checked.as_ref().map(|(program, wanted)| (program.as_str(), *wanted)), page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListModuleCatalogUseCase::list"))
            .await?;

        Ok(ListModuleCatalogOutput {
            module_catalog,
            next_page_token,
            total_size,
        })
    }
}
