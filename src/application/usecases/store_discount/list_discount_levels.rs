use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::{DiscountLevels, StoreDiscountRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct ListDiscountLevelsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    store_discount_repository: Arc<dyn StoreDiscountRepository>,
}

impl ListDiscountLevelsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self { program_repository, store_discount_repository }
    }
}

#[derive(Debug)]
pub struct ListDiscountLevelsInput {
    pub program: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<DiscountLevels, ListDiscountLevelsInput> for ListDiscountLevelsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListDiscountLevelsInput>) -> Result<DiscountLevels, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListDiscountLevelsUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListDiscountLevelsUseCase::get_program"))
            .await?;

        self.store_discount_repository
            .levels(program.id)
            .instrument(info_span!("ListDiscountLevelsUseCase::levels"))
            .await
    }
}
