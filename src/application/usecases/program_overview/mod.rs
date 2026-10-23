use crate::domain::program::ProgramRepository;
use crate::domain::program_overview::ProgramOverviewRepository;
use std::sync::Arc;

pub mod get_overview;

use get_overview::GetOverviewUseCase;

pub struct ProgramOverviewUseCases {
    pub get_overview: GetOverviewUseCase,
}

impl ProgramOverviewUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        overview_repository: Arc<dyn ProgramOverviewRepository>,
    ) -> Self {
        Self { get_overview: GetOverviewUseCase::new(program_repository, overview_repository) }
    }
}
