use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::{StoreDiscountRepository, StoreDiscountRow};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct GetStoreDiscountUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    store_discount_repository: Arc<dyn StoreDiscountRepository>,
}

impl GetStoreDiscountUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self { program_repository, store_discount_repository }
    }
}

#[derive(Debug)]
pub struct GetStoreDiscountInput {
    pub program: ResourceIdentifier,
    pub enrollment: Uuid,
}

#[async_trait::async_trait]
impl UseCase<StoreDiscountRow, GetStoreDiscountInput> for GetStoreDiscountUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetStoreDiscountInput>) -> Result<StoreDiscountRow, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetStoreDiscountUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetStoreDiscountUseCase::get_program"))
            .await?;

        self.store_discount_repository
            .get(program.id, input.enrollment)
            .instrument(info_span!("GetStoreDiscountUseCase::get"))
            .await
    }
}
