use crate::domain::program::ProgramRepository;
use crate::domain::approval_step::ApprovalStepRepository;
use std::sync::Arc;

pub mod batch_update_approval_steps;
pub mod create_approval_step;
pub mod delete_approval_step;
pub mod get_approval_step;
pub mod list_approval_steps;
pub mod update_approval_step;

use batch_update_approval_steps::BatchUpdateApprovalStepsUseCase;
use create_approval_step::CreateApprovalStepUseCase;
use delete_approval_step::DeleteApprovalStepUseCase;
use get_approval_step::GetApprovalStepUseCase;
use list_approval_steps::ListApprovalStepsUseCase;
use update_approval_step::UpdateApprovalStepUseCase;

pub struct ApprovalStepUseCases {
    pub list_approval_steps: ListApprovalStepsUseCase,
    pub get_approval_step: GetApprovalStepUseCase,
    pub create_approval_step: CreateApprovalStepUseCase,
    pub update_approval_step: UpdateApprovalStepUseCase,
    pub delete_approval_step: DeleteApprovalStepUseCase,
    pub batch_update_approval_steps: BatchUpdateApprovalStepsUseCase,
}

impl ApprovalStepUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self {
            list_approval_steps: ListApprovalStepsUseCase::new(program_repository.clone(), approval_step_repository.clone()),
            get_approval_step: GetApprovalStepUseCase::new(program_repository.clone(), approval_step_repository.clone()),
            create_approval_step: CreateApprovalStepUseCase::new(program_repository.clone(), approval_step_repository.clone()),
            update_approval_step: UpdateApprovalStepUseCase::new(program_repository.clone(), approval_step_repository.clone()),
            delete_approval_step: DeleteApprovalStepUseCase::new(program_repository.clone(), approval_step_repository.clone()),
            batch_update_approval_steps: BatchUpdateApprovalStepsUseCase::new(program_repository, approval_step_repository),
        }
    }
}
