use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::{StoreDiscountFilter, StoreDiscountRepository, StoreDiscountRow};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListStoreDiscountsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    store_discount_repository: Arc<dyn StoreDiscountRepository>,
}

impl ListStoreDiscountsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self { program_repository, store_discount_repository }
    }
}

#[derive(Debug)]
pub struct ListStoreDiscountsInput {
    pub program: ResourceIdentifier,
    pub website_id: Option<String>,
    pub store: Option<String>,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListStoreDiscountsOutput {
    pub rows: Vec<StoreDiscountRow>,
    pub next_page_token: Option<Uuid>,
    /// How many enrolled stores match the filter.
    pub total_size: i64,
}

#[async_trait::async_trait]
impl UseCase<ListStoreDiscountsOutput, ListStoreDiscountsInput> for ListStoreDiscountsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListStoreDiscountsInput>) -> Result<ListStoreDiscountsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListStoreDiscountsUseCase"))?;

        let filter = StoreDiscountFilter {
            website_id: input.website_id.map(|w| w.trim().to_string()).filter(|w| !w.is_empty()),
            store_prefix: input.store.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        };
        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListStoreDiscountsUseCase::get_program"))
            .await?;

        let (rows, next_page_token, total_size) = self
            .store_discount_repository
            .list(program.id, &filter, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListStoreDiscountsUseCase::list"))
            .await?;

        Ok(ListStoreDiscountsOutput { rows, next_page_token, total_size })
    }
}
