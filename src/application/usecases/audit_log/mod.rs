use crate::domain::program::ProgramRepository;
use crate::domain::program_overview::AuditLogRepository;
use std::sync::Arc;

pub mod list_audit_logs;

use list_audit_logs::ListAuditLogsUseCase;

pub struct AuditLogUseCases {
    pub list_audit_logs: ListAuditLogsUseCase,
}

impl AuditLogUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        audit_log_repository: Arc<dyn AuditLogRepository>,
    ) -> Self {
        Self { list_audit_logs: ListAuditLogsUseCase::new(program_repository, audit_log_repository) }
    }
}
