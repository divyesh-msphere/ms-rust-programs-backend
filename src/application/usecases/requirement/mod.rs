use crate::domain::program::ProgramRepository;
use crate::domain::requirement::RequirementRepository;
use std::sync::Arc;

pub mod batch_update_requirements;
pub mod create_requirement;
pub mod delete_requirement;
pub mod get_requirement;
pub mod list_requirements;
pub mod update_requirement;

use batch_update_requirements::BatchUpdateRequirementsUseCase;
use create_requirement::CreateRequirementUseCase;
use delete_requirement::DeleteRequirementUseCase;
use get_requirement::GetRequirementUseCase;
use list_requirements::ListRequirementsUseCase;
use update_requirement::UpdateRequirementUseCase;

pub struct RequirementUseCases {
    pub list_requirements: ListRequirementsUseCase,
    pub get_requirement: GetRequirementUseCase,
    pub create_requirement: CreateRequirementUseCase,
    pub update_requirement: UpdateRequirementUseCase,
    pub delete_requirement: DeleteRequirementUseCase,
    pub batch_update_requirements: BatchUpdateRequirementsUseCase,
}

impl RequirementUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self {
            list_requirements: ListRequirementsUseCase::new(program_repository.clone(), requirement_repository.clone()),
            get_requirement: GetRequirementUseCase::new(program_repository.clone(), requirement_repository.clone()),
            create_requirement: CreateRequirementUseCase::new(program_repository.clone(), requirement_repository.clone()),
            update_requirement: UpdateRequirementUseCase::new(program_repository.clone(), requirement_repository.clone()),
            delete_requirement: DeleteRequirementUseCase::new(program_repository.clone(), requirement_repository.clone()),
            batch_update_requirements: BatchUpdateRequirementsUseCase::new(program_repository, requirement_repository),
        }
    }
}
