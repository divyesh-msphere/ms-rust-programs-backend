use crate::domain::application::ApplicationRepository;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;

pub mod create_application;
pub mod get_application;
pub mod list_applications;
pub mod submit_application;
pub mod withdraw_application;

use create_application::CreateApplicationUseCase;
use get_application::GetApplicationUseCase;
use list_applications::ListApplicationsUseCase;
use submit_application::SubmitApplicationUseCase;
use withdraw_application::WithdrawApplicationUseCase;

pub struct ApplicationUseCases {
    pub list_applications: ListApplicationsUseCase,
    pub get_application: GetApplicationUseCase,
    pub create_application: CreateApplicationUseCase,
    pub submit_application: SubmitApplicationUseCase,
    pub withdraw_application: WithdrawApplicationUseCase,
}

impl ApplicationUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, application_repository: Arc<dyn ApplicationRepository>) -> Self {
        Self {
            list_applications: ListApplicationsUseCase::new(program_repository.clone(), application_repository.clone()),
            get_application: GetApplicationUseCase::new(program_repository.clone(), application_repository.clone()),
            create_application: CreateApplicationUseCase::new(program_repository.clone(), application_repository.clone()),
            submit_application: SubmitApplicationUseCase::new(program_repository.clone(), application_repository.clone()),
            withdraw_application: WithdrawApplicationUseCase::new(program_repository, application_repository),
        }
    }
}
