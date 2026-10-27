use crate::domain::program::ProgramRepository;
use crate::domain::signoff::SignoffRepository;
use std::sync::Arc;

pub mod create_signoff;
pub mod get_signoff;
pub mod list_signoffs;

use create_signoff::CreateSignoffUseCase;
use get_signoff::GetSignoffUseCase;
use list_signoffs::ListSignoffsUseCase;

pub struct SignoffUseCases {
    pub list_signoffs: ListSignoffsUseCase,
    pub get_signoff: GetSignoffUseCase,
    pub create_signoff: CreateSignoffUseCase,
}

impl SignoffUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, signoff_repository: Arc<dyn SignoffRepository>) -> Self {
        Self {
            list_signoffs: ListSignoffsUseCase::new(program_repository.clone(), signoff_repository.clone()),
            get_signoff: GetSignoffUseCase::new(program_repository.clone(), signoff_repository.clone()),
            create_signoff: CreateSignoffUseCase::new(program_repository, signoff_repository),
        }
    }
}
