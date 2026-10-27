use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::enrollment::{Enrollment, EnrollmentRepository, UpdateEnrollment, validate_update};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct UpdateEnrollmentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    enrollment_repository: Arc<dyn EnrollmentRepository>,
}

impl UpdateEnrollmentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, enrollment_repository: Arc<dyn EnrollmentRepository>) -> Self {
        Self { program_repository, enrollment_repository }
    }
}

#[derive(Debug)]
pub struct UpdateEnrollmentInput {
    pub program: ResourceIdentifier,
    pub enrollment: Uuid,
    pub status: Option<String>,
    pub customer_group: Option<String>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Enrollment, UpdateEnrollmentInput> for UpdateEnrollmentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateEnrollmentInput>) -> Result<Enrollment, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateEnrollmentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateEnrollmentUseCase::get_program"))
            .await?;

        let update = UpdateEnrollment { status: input.status, customer_group: input.customer_group, actor: input.actor };
        validate_update(&update)?;

        self.enrollment_repository
            .update(program.id, input.enrollment, update)
            .instrument(info_span!("UpdateEnrollmentUseCase::update"))
            .await
    }
}
