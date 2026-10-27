use crate::adapters::services::postgres::application::application_repository::ApplicationRepositoryPostgresImpl;
use crate::adapters::services::postgres::approval_step::approval_step_repository::ApprovalStepRepositoryPostgresImpl;
use crate::adapters::services::postgres::audit_log::audit_log_repository::AuditLogRepositoryPostgresImpl;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::document::document_repository::DocumentRepositoryPostgresImpl;
use crate::adapters::services::postgres::enrollment::enrollment_repository::EnrollmentRepositoryPostgresImpl;
use crate::adapters::services::postgres::module_definition::module_definition_repository::ModuleDefinitionRepositoryPostgresImpl;
use crate::adapters::services::postgres::program::program_repository::ProgramRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_field_value::program_field_value_repository::ProgramFieldValueRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_overview::program_overview_repository::ProgramOverviewRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_module::program_module_repository::ProgramModuleRepositoryPostgresImpl;
use crate::adapters::services::postgres::signoff::signoff_repository::SignoffRepositoryPostgresImpl;
use crate::adapters::services::postgres::requirement::requirement_repository::RequirementRepositoryPostgresImpl;
use crate::adapters::services::postgres::store_discount::store_discount_repository::StoreDiscountRepositoryPostgresImpl;
use crate::adapters::services::postgres::publication::publication_repository::PublicationRepositoryPostgresImpl;
use crate::adapters::services::postgres::rule_set::rule_set_repository::RuleSetRepositoryPostgresImpl;
use crate::config::ApplicationConfig;
use crate::infrastructure::feature_switch;
use crate::domain::application::ApplicationRepository;
use crate::domain::approval_step::ApprovalStepRepository;
use crate::domain::document::DocumentRepository;
use crate::domain::enrollment::EnrollmentRepository;
use crate::domain::module_definition::ModuleDefinitionRepository;
use crate::domain::program::ProgramRepository;
use crate::domain::program_field_value::ProgramFieldValueRepository;
use crate::domain::program_module::ProgramModuleRepository;
use crate::domain::program_overview::{AuditLogRepository, ProgramOverviewRepository};
use crate::domain::publication::PublicationRepository;
use crate::domain::requirement::RequirementRepository;
use crate::domain::rule_set::RuleSetRepository;
use crate::domain::signoff::SignoffRepository;
use crate::domain::store_discount::StoreDiscountRepository;
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
    pub signoff_repository: Arc<dyn SignoffRepository>,
    pub enrollment_repository: Arc<dyn EnrollmentRepository>,
    pub store_discount_repository: Arc<dyn StoreDiscountRepository>,
    pub rule_set_repository: Arc<dyn RuleSetRepository>,
    pub publication_repository: Arc<dyn PublicationRepository>,
    pub document_repository: Arc<dyn DocumentRepository>,
}

impl Repositories {
    pub async fn new(app_config: Arc<ApplicationConfig>) -> Self {
        let postgres_connection = Arc::new(PostgresDbConnection::new(&app_config.postgres));
        // Show or hide the parked modules' pages before anything reads the module tree.
        feature_switch::apply(postgres_connection.get_pool(), &app_config.features)
            .await
            .expect("Failed to apply [features] (apply migrations/20261027100000_parked_modules.up.sql first)");

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
            application_repository: Arc::new(ApplicationRepositoryPostgresImpl::new(postgres_connection.clone())),
            signoff_repository: Arc::new(SignoffRepositoryPostgresImpl::new(postgres_connection.clone())),
            enrollment_repository: Arc::new(EnrollmentRepositoryPostgresImpl::new(postgres_connection.clone())),
            store_discount_repository: Arc::new(StoreDiscountRepositoryPostgresImpl::new(postgres_connection.clone())),
            rule_set_repository: Arc::new(RuleSetRepositoryPostgresImpl::new(postgres_connection.clone())),
            publication_repository: Arc::new(PublicationRepositoryPostgresImpl::new(postgres_connection.clone())),
            document_repository: Arc::new(DocumentRepositoryPostgresImpl::new(postgres_connection)),
        }
    }
}
