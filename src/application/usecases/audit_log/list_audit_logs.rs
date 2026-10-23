use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::program_overview::{AuditCursor, AuditEntry, AuditLogRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;
const MAX_ACTION_FILTERS: usize = 20;

pub struct ListAuditLogsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    audit_log_repository: Arc<dyn AuditLogRepository>,
}

impl ListAuditLogsUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        audit_log_repository: Arc<dyn AuditLogRepository>,
    ) -> Self {
        Self { program_repository, audit_log_repository }
    }
}

#[derive(Debug)]
pub struct ListAuditLogsInput {
    pub program: ResourceIdentifier,
    pub page_size: Option<i64>,
    pub page_token: Option<String>,
    /// Only these actions (`created`, `updated`, `values_updated`, ...). Empty means all.
    pub actions: Vec<String>,
}

#[derive(Debug)]
pub struct ListAuditLogsOutput {
    pub entries: Vec<AuditEntry>,
    pub next_page_token: Option<String>,
}

#[async_trait::async_trait]
impl UseCase<ListAuditLogsOutput, ListAuditLogsInput> for ListAuditLogsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListAuditLogsInput>) -> Result<ListAuditLogsOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in ListAuditLogsUseCase")
        })?;

        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }
        if input.actions.len() > MAX_ACTION_FILTERS {
            return Err(ApplicationError::validation_error(&format!("at most {MAX_ACTION_FILTERS} actions")));
        }
        let cursor = input.page_token.as_deref().map(AuditCursor::decode).transpose()?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListAuditLogsUseCase::get_program"))
            .await?;

        let (entries, next) = self
            .audit_log_repository
            .list(program.id, page_size.min(MAX_PAGE_SIZE), cursor, &input.actions)
            .instrument(info_span!("ListAuditLogsUseCase::list"))
            .await?;

        Ok(ListAuditLogsOutput { entries, next_page_token: next.map(|c| c.encode()) })
    }
}
