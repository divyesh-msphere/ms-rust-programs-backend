use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;
use crate::domain::program_query::{parse_filter, parse_search};
use crate::domain::program::{ProgramFilter, ProgramListItem, ProgramListSummary, ProgramRepository};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListProgramsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
}

impl ListProgramsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>) -> Self {
        Self { program_repository }
    }
}

#[derive(Debug)]
pub struct ListProgramsInput {
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
    pub show_deleted: bool,
    pub show_archived: bool,
    /// The search box: `program=abc` (see `program_query`).
    pub search: Option<String>,
    /// `program.not_contains=abc,description.is_empty,domain=[a,b],status=active,level=organization`.
    pub filter: Option<String>,
}

#[derive(Debug)]
pub struct ListProgramsOutput {
    pub programs: Vec<ProgramListItem>,
    pub summary: ProgramListSummary,
    pub next_page_token: Option<Uuid>,
    pub total_size: i64,
}

#[async_trait::async_trait]
impl UseCase<ListProgramsOutput, ListProgramsInput> for ListProgramsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListProgramsInput>) -> Result<ListProgramsOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListProgramsUseCase")
        })?;

        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let list = parse_filter(input.filter.as_deref())?;
        let filter = ProgramFilter {
            show_deleted: input.show_deleted,
            show_archived: input.show_archived,
            search: parse_search(input.search.as_deref())?.into_iter().chain(list.text.clone()).collect(),
            list,
        };
        let (programs, next_page_token, total_size) = self
            .program_repository
            .list(page_size.min(MAX_PAGE_SIZE), input.page_token, &filter)
            .instrument(info_span!("ListProgramsUseCase::list"))
            .await?;

        let ids: Vec<Uuid> = programs.iter().map(|p| p.id).collect();
        let (mut stats, summary) = tokio::try_join!(
            self.program_repository.list_stats(&ids),
            self.program_repository.list_summary(input.show_deleted, input.show_archived),
        )?;
        let programs = programs
            .into_iter()
            .map(|program| {
                let stats = stats.remove(&program.id).unwrap_or_default();
                ProgramListItem { program, stats }
            })
            .collect();

        Ok(ListProgramsOutput { programs, summary, next_page_token, total_size })
    }
}
