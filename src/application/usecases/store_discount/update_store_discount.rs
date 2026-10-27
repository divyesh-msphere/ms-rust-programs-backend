use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::{DiscountChange, StoreDiscountRepository, StoreDiscountRow};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct UpdateStoreDiscountUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    store_discount_repository: Arc<dyn StoreDiscountRepository>,
}

impl UpdateStoreDiscountUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self { program_repository, store_discount_repository }
    }
}

#[derive(Debug)]
pub struct UpdateStoreDiscountInput {
    pub program: ResourceIdentifier,
    pub enrollment: Uuid,
    pub change: DiscountChange,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<StoreDiscountRow, UpdateStoreDiscountInput> for UpdateStoreDiscountUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateStoreDiscountInput>) -> Result<StoreDiscountRow, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateStoreDiscountUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateStoreDiscountUseCase::get_program"))
            .await?;

        self.store_discount_repository
            .update_store(program.id, input.enrollment, &input.change, &input.actor)
            .instrument(info_span!("UpdateStoreDiscountUseCase::update_store"))
            .await
    }
}
