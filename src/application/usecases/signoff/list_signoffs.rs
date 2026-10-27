use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::signoff::{Signoff, SignoffRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct ListSignoffsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    signoff_repository: Arc<dyn SignoffRepository>,
}

impl ListSignoffsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, signoff_repository: Arc<dyn SignoffRepository>) -> Self {
        Self { program_repository, signoff_repository }
    }
}

#[derive(Debug)]
pub struct ListSignoffsInput {
    pub program: ResourceIdentifier,
    pub application: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Vec<Signoff>, ListSignoffsInput> for ListSignoffsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListSignoffsInput>) -> Result<Vec<Signoff>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListSignoffsUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListSignoffsUseCase::get_program"))
            .await?;

        self.signoff_repository
            .list(program.id, input.application)
            .instrument(info_span!("ListSignoffsUseCase::list"))
            .await
    }
}
