use crate::domain::program::ProgramRepository;
use crate::domain::store_discount::StoreDiscountRepository;
use std::sync::Arc;

pub mod batch_update_store_discounts;
pub mod get_store_discount;
pub mod list_discount_levels;
pub mod list_store_discounts;
pub mod update_discount_level;
pub mod update_store_discount;

use batch_update_store_discounts::BatchUpdateStoreDiscountsUseCase;
use get_store_discount::GetStoreDiscountUseCase;
use list_discount_levels::ListDiscountLevelsUseCase;
use list_store_discounts::ListStoreDiscountsUseCase;
use update_discount_level::UpdateDiscountLevelUseCase;
use update_store_discount::UpdateStoreDiscountUseCase;

pub struct StoreDiscountUseCases {
    pub list_store_discounts: ListStoreDiscountsUseCase,
    pub get_store_discount: GetStoreDiscountUseCase,
    pub update_store_discount: UpdateStoreDiscountUseCase,
    pub batch_update_store_discounts: BatchUpdateStoreDiscountsUseCase,
    pub list_discount_levels: ListDiscountLevelsUseCase,
    pub update_discount_level: UpdateDiscountLevelUseCase,
}

impl StoreDiscountUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, store_discount_repository: Arc<dyn StoreDiscountRepository>) -> Self {
        Self {
            list_store_discounts: ListStoreDiscountsUseCase::new(program_repository.clone(), store_discount_repository.clone()),
            get_store_discount: GetStoreDiscountUseCase::new(program_repository.clone(), store_discount_repository.clone()),
            update_store_discount: UpdateStoreDiscountUseCase::new(program_repository.clone(), store_discount_repository.clone()),
            batch_update_store_discounts: BatchUpdateStoreDiscountsUseCase::new(program_repository.clone(), store_discount_repository.clone()),
            list_discount_levels: ListDiscountLevelsUseCase::new(program_repository.clone(), store_discount_repository.clone()),
            update_discount_level: UpdateDiscountLevelUseCase::new(program_repository, store_discount_repository),
        }
    }
}
