use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::enrollment::{ACTIVE, BatchOutcome, EnrollmentRepository, MAX_BATCH, NewEnrollment, validate_new};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct BatchCreateEnrollmentsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    enrollment_repository: Arc<dyn EnrollmentRepository>,
}

impl BatchCreateEnrollmentsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, enrollment_repository: Arc<dyn EnrollmentRepository>) -> Self {
        Self { program_repository, enrollment_repository }
    }
}

#[derive(Debug)]
pub struct BatchCreateEnrollmentsInput {
    pub program: ResourceIdentifier,
    pub items: Vec<BatchItem>,
    pub actor: String,
}

#[derive(Debug)]
pub struct BatchItem {
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub customer_group: Option<String>,
    pub status: Option<String>,
}

/// What happened to one item, in the order the items were sent.
#[derive(Debug)]
pub struct BatchResult {
    pub store_id: String,
    pub website_id: String,
    pub outcome: BatchOutcome,
}

#[async_trait::async_trait]
impl UseCase<Vec<BatchResult>, BatchCreateEnrollmentsInput> for BatchCreateEnrollmentsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<BatchCreateEnrollmentsInput>) -> Result<Vec<BatchResult>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in BatchCreateEnrollmentsUseCase"))?;

        if input.items.is_empty() {
            return Err(ApplicationError::validation_error("items must list at least one store"));
        }
        if input.items.len() > MAX_BATCH {
            return Err(ApplicationError::validation_error(&format!("items must have at most {MAX_BATCH} stores")));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("BatchCreateEnrollmentsUseCase::get_program"))
            .await?;

        // Each item is checked on its own: a bad one is reported, the rest still go in.
        let mut results: Vec<BatchResult> = Vec::with_capacity(input.items.len());
        let mut valid: Vec<NewEnrollment> = Vec::new();
        let mut valid_at: Vec<usize> = Vec::new();
        for (index, item) in input.items.into_iter().enumerate() {
            let new_enrollment = NewEnrollment {
                id: None,
                program_id: program.id,
                application_id: None,
                store_id: item.store_id.clone(),
                store_name: item.store_name,
                website_id: item.website_id.clone(),
                website_name: item.website_name,
                customer_group: item.customer_group,
                status: item.status.unwrap_or_else(|| ACTIVE.to_string()),
                actor: input.actor.clone(),
            };
            match validate_new(&new_enrollment) {
                Ok(()) => {
                    valid_at.push(index);
                    valid.push(new_enrollment);
                    results.push(BatchResult { store_id: item.store_id, website_id: item.website_id, outcome: BatchOutcome::AlreadyEnrolled });
                }
                Err(err) => results.push(BatchResult {
                    store_id: item.store_id,
                    website_id: item.website_id,
                    outcome: BatchOutcome::Failed(err.message().to_string()),
                }),
            }
        }

        let outcomes = self
            .enrollment_repository
            .batch_create(program.id, valid, &input.actor)
            .instrument(info_span!("BatchCreateEnrollmentsUseCase::batch_create"))
            .await?;
        for (index, outcome) in valid_at.into_iter().zip(outcomes) {
            results[index].outcome = outcome;
        }
        Ok(results)
    }
}
