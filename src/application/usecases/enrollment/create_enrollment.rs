use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::enrollment::{ACTIVE, Enrollment, EnrollmentRepository, NewEnrollment, validate_new};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct CreateEnrollmentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    enrollment_repository: Arc<dyn EnrollmentRepository>,
}

impl CreateEnrollmentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, enrollment_repository: Arc<dyn EnrollmentRepository>) -> Self {
        Self { program_repository, enrollment_repository }
    }
}

#[derive(Debug)]
pub struct CreateEnrollmentInput {
    pub program: ResourceIdentifier,
    pub id: Option<Uuid>,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub customer_group: Option<String>,
    /// `pending` or `active` (default).
    pub status: Option<String>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Enrollment, CreateEnrollmentInput> for CreateEnrollmentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateEnrollmentInput>) -> Result<Enrollment, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateEnrollmentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateEnrollmentUseCase::get_program"))
            .await?;

        let new_enrollment = NewEnrollment {
            id: input.id,
            program_id: program.id,
            application_id: None,
            store_id: input.store_id,
            store_name: input.store_name,
            website_id: input.website_id,
            website_name: input.website_name,
            customer_group: input.customer_group,
            status: input.status.unwrap_or_else(|| ACTIVE.to_string()),
            actor: input.actor,
        };
        validate_new(&new_enrollment)?;

        self.enrollment_repository
            .create(new_enrollment)
            .instrument(info_span!("CreateEnrollmentUseCase::create"))
            .await
    }
}
