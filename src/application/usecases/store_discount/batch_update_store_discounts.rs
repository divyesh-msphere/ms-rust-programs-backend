use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::{DiscountChange, MAX_BATCH, StoreDiscountRepository, StoreDiscountRow};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct BatchUpdateStoreDiscountsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    store_discount_repository: Arc<dyn StoreDiscountRepository>,
}

impl BatchUpdateStoreDiscountsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self { program_repository, store_discount_repository }
    }
}

#[derive(Debug)]
pub struct BatchUpdateStoreDiscountsInput {
    pub program: ResourceIdentifier,
    pub ids: Vec<Uuid>,
    pub change: DiscountChange,
    pub actor: String,
}

#[derive(Debug)]
pub struct BatchUpdateStoreDiscountsOutput {
    /// How many stores really changed (one that already had these values is not a change).
    pub updated: usize,
    pub rows: Vec<StoreDiscountRow>,
}

#[async_trait::async_trait]
impl UseCase<BatchUpdateStoreDiscountsOutput, BatchUpdateStoreDiscountsInput> for BatchUpdateStoreDiscountsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<BatchUpdateStoreDiscountsInput>) -> Result<BatchUpdateStoreDiscountsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in BatchUpdateStoreDiscountsUseCase"))?;

        if input.ids.is_empty() {
            return Err(ApplicationError::validation_error("ids must list at least one store"));
        }
        if input.ids.len() > MAX_BATCH {
            return Err(ApplicationError::validation_error(&format!("ids must have at most {MAX_BATCH} stores")));
        }
        let mut seen = std::collections::HashSet::new();
        if let Some(repeated) = input.ids.iter().find(|id| !seen.insert(**id)) {
            return Err(ApplicationError::validation_error(&format!("store {repeated} is listed twice")));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("BatchUpdateStoreDiscountsUseCase::get_program"))
            .await?;

        let (updated, rows) = self
            .store_discount_repository
            .batch_update_stores(program.id, &input.ids, &input.change, &input.actor)
            .instrument(info_span!("BatchUpdateStoreDiscountsUseCase::batch_update_stores"))
            .await?;

        Ok(BatchUpdateStoreDiscountsOutput { updated, rows })
    }
}
