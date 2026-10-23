use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::application::{Application, ApplicationRepository, parse_status_filter};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListApplicationsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    application_repository: Arc<dyn ApplicationRepository>,
}

impl ListApplicationsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, application_repository: Arc<dyn ApplicationRepository>) -> Self {
        Self { program_repository, application_repository }
    }
}

#[derive(Debug)]
pub struct ListApplicationsInput {
    pub program: ResourceIdentifier,
    pub status: Option<String>,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListApplicationsOutput {
    pub applications: Vec<Application>,
    pub next_page_token: Option<Uuid>,
}

#[async_trait::async_trait]
impl UseCase<ListApplicationsOutput, ListApplicationsInput> for ListApplicationsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListApplicationsInput>) -> Result<ListApplicationsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListApplicationsUseCase"))?;

        let statuses = parse_status_filter(input.status.as_deref())?;
        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListApplicationsUseCase::get_program"))
            .await?;

        let (applications, next_page_token) = self
            .application_repository
            .list(program.id, &statuses, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListApplicationsUseCase::list"))
            .await?;

        Ok(ListApplicationsOutput { applications, next_page_token })
    }
}
