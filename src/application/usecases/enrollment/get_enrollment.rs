use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::enrollment::{Enrollment, EnrollmentRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct GetEnrollmentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    enrollment_repository: Arc<dyn EnrollmentRepository>,
}

impl GetEnrollmentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, enrollment_repository: Arc<dyn EnrollmentRepository>) -> Self {
        Self { program_repository, enrollment_repository }
    }
}

#[derive(Debug)]
pub struct GetEnrollmentInput {
    pub program: ResourceIdentifier,
    pub enrollment: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Enrollment, GetEnrollmentInput> for GetEnrollmentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetEnrollmentInput>) -> Result<Enrollment, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetEnrollmentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetEnrollmentUseCase::get_program"))
            .await?;

        self.enrollment_repository
            .get(program.id, input.enrollment)
            .instrument(info_span!("GetEnrollmentUseCase::get"))
            .await
    }
}
