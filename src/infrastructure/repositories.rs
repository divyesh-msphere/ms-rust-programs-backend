use crate::adapters::services::postgres::application::application_repository::ApplicationRepositoryPostgresImpl;
use crate::adapters::services::postgres::approval_step::approval_step_repository::ApprovalStepRepositoryPostgresImpl;
use crate::adapters::services::postgres::audit_log::audit_log_repository::AuditLogRepositoryPostgresImpl;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::module_definition::module_definition_repository::ModuleDefinitionRepositoryPostgresImpl;
use crate::adapters::services::postgres::program::program_repository::ProgramRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_field_value::program_field_value_repository::ProgramFieldValueRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_overview::program_overview_repository::ProgramOverviewRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_module::program_module_repository::ProgramModuleRepositoryPostgresImpl;
use crate::adapters::services::postgres::requirement::requirement_repository::RequirementRepositoryPostgresImpl;
use crate::config::ApplicationConfig;
use crate::domain::application::ApplicationRepository;
use crate::domain::approval_step::ApprovalStepRepository;
use crate::domain::module_definition::ModuleDefinitionRepository;
use crate::domain::program::ProgramRepository;
use crate::domain::program_field_value::ProgramFieldValueRepository;
use crate::domain::program_module::ProgramModuleRepository;
use crate::domain::program_overview::{AuditLogRepository, ProgramOverviewRepository};
use crate::domain::requirement::RequirementRepository;
use std::sync::Arc;

pub struct Repositories {
    pub module_definition_repository: Arc<dyn ModuleDefinitionRepository>,
    pub program_repository: Arc<dyn ProgramRepository>,
    pub program_module_repository: Arc<dyn ProgramModuleRepository>,
    pub program_field_value_repository: Arc<dyn ProgramFieldValueRepository>,
    pub program_overview_repository: Arc<dyn ProgramOverviewRepository>,
    pub audit_log_repository: Arc<dyn AuditLogRepository>,
    pub requirement_repository: Arc<dyn RequirementRepository>,
    pub approval_step_repository: Arc<dyn ApprovalStepRepository>,
    pub application_repository: Arc<dyn ApplicationRepository>,
}

impl Repositories {
    pub async fn new(app_config: Arc<ApplicationConfig>) -> Self {
        let postgres_connection = Arc::new(PostgresDbConnection::new(&app_config.postgres));

        Self {
            module_definition_repository: Arc::new(ModuleDefinitionRepositoryPostgresImpl::new(
                postgres_connection.clone(),
            )),
            program_repository: Arc::new(ProgramRepositoryPostgresImpl::new(postgres_connection.clone())),
            program_module_repository: Arc::new(ProgramModuleRepositoryPostgresImpl::new(postgres_connection.clone())),
            program_field_value_repository: Arc::new(ProgramFieldValueRepositoryPostgresImpl::new(postgres_connection.clone())),
            program_overview_repository: Arc::new(ProgramOverviewRepositoryPostgresImpl::new(postgres_connection.clone())),
            audit_log_repository: Arc::new(AuditLogRepositoryPostgresImpl::new(postgres_connection.clone())),
            requirement_repository: Arc::new(RequirementRepositoryPostgresImpl::new(postgres_connection.clone())),
            approval_step_repository: Arc::new(ApprovalStepRepositoryPostgresImpl::new(postgres_connection.clone())),
            application_repository: Arc::new(ApplicationRepositoryPostgresImpl::new(postgres_connection)),
        }
    }
}
