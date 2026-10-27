use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::enrollment::{Enrollment, EnrollmentFilter, EnrollmentRepository, parse_status_filter};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListEnrollmentsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    enrollment_repository: Arc<dyn EnrollmentRepository>,
}

impl ListEnrollmentsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, enrollment_repository: Arc<dyn EnrollmentRepository>) -> Self {
        Self { program_repository, enrollment_repository }
    }
}

#[derive(Debug)]
pub struct ListEnrollmentsInput {
    pub program: ResourceIdentifier,
    pub website_id: Option<String>,
    pub status: Option<String>,
    pub store: Option<String>,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListEnrollmentsOutput {
    pub enrollments: Vec<Enrollment>,
    pub next_page_token: Option<Uuid>,
    /// How many enrollments match the filter (the "N STORES" count).
    pub total_size: i64,
}

#[async_trait::async_trait]
impl UseCase<ListEnrollmentsOutput, ListEnrollmentsInput> for ListEnrollmentsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListEnrollmentsInput>) -> Result<ListEnrollmentsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListEnrollmentsUseCase"))?;

        let filter = EnrollmentFilter {
            website_id: input.website_id.map(|w| w.trim().to_string()).filter(|w| !w.is_empty()),
            statuses: parse_status_filter(input.status.as_deref())?,
            store_prefix: input.store.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        };
        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListEnrollmentsUseCase::get_program"))
            .await?;

        let (enrollments, next_page_token, total_size) = self
            .enrollment_repository
            .list(program.id, &filter, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListEnrollmentsUseCase::list"))
            .await?;

        Ok(ListEnrollmentsOutput { enrollments, next_page_token, total_size })
    }
}
