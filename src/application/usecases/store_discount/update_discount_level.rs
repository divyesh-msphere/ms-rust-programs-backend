use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::{DiscountChange, DiscountConfig, LevelTarget, StoreDiscountRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct UpdateDiscountLevelUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    store_discount_repository: Arc<dyn StoreDiscountRepository>,
}

impl UpdateDiscountLevelUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self { program_repository, store_discount_repository }
    }
}

#[derive(Debug)]
pub struct UpdateDiscountLevelInput {
    pub program: ResourceIdentifier,
    pub target: LevelTarget,
    pub change: DiscountChange,
    pub actor: String,
}

#[derive(Debug)]
pub struct UpdateDiscountLevelOutput {
    pub target: LevelTarget,
    /// What that level sets afterwards.
    pub discount: DiscountConfig,
}

#[async_trait::async_trait]
impl UseCase<UpdateDiscountLevelOutput, UpdateDiscountLevelInput> for UpdateDiscountLevelUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateDiscountLevelInput>) -> Result<UpdateDiscountLevelOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateDiscountLevelUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateDiscountLevelUseCase::get_program"))
            .await?;

        let discount = self
            .store_discount_repository
            .update_level(program.id, &input.target, &input.change, &input.actor)
            .instrument(info_span!("UpdateDiscountLevelUseCase::update_level"))
            .await?;

        Ok(UpdateDiscountLevelOutput { target: input.target, discount })
    }
}
