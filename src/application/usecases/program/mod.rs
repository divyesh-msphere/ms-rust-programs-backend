use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;

pub mod create_program;
pub mod delete_program;
pub mod get_program;
pub mod list_programs;
pub mod undelete_program;
pub mod update_program;

use create_program::CreateProgramUseCase;
use delete_program::DeleteProgramUseCase;
use get_program::GetProgramUseCase;
use list_programs::ListProgramsUseCase;
use undelete_program::UndeleteProgramUseCase;
use update_program::UpdateProgramUseCase;

pub struct ProgramUseCases {
    pub list_programs: ListProgramsUseCase,
    pub get_program: GetProgramUseCase,
    pub create_program: CreateProgramUseCase,
    pub update_program: UpdateProgramUseCase,
    pub delete_program: DeleteProgramUseCase,
    pub undelete_program: UndeleteProgramUseCase,
}

impl ProgramUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, library_catalog: Arc<dyn LibraryCatalog>) -> Self {
        Self {
            list_programs: ListProgramsUseCase::new(program_repository.clone()),
            get_program: GetProgramUseCase::new(program_repository.clone()),
            create_program: CreateProgramUseCase::new(program_repository.clone(), library_catalog.clone()),
            update_program: UpdateProgramUseCase::new(program_repository.clone(), library_catalog),
            delete_program: DeleteProgramUseCase::new(program_repository.clone()),
            undelete_program: UndeleteProgramUseCase::new(program_repository),
        }
    }
}
