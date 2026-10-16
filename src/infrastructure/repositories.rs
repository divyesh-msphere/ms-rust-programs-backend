use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::module_definition::module_definition_repository::ModuleDefinitionRepositoryPostgresImpl;
use crate::adapters::services::postgres::program::program_repository::ProgramRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_field_value::program_field_value_repository::ProgramFieldValueRepositoryPostgresImpl;
use crate::adapters::services::postgres::program_module::program_module_repository::ProgramModuleRepositoryPostgresImpl;
use crate::config::ApplicationConfig;
use crate::domain::module_definition::ModuleDefinitionRepository;
use crate::domain::program::ProgramRepository;
use crate::domain::program_field_value::ProgramFieldValueRepository;
use crate::domain::program_module::ProgramModuleRepository;
use std::sync::Arc;

pub struct Repositories {
    pub module_definition_repository: Arc<dyn ModuleDefinitionRepository>,
    pub program_repository: Arc<dyn ProgramRepository>,
    pub program_module_repository: Arc<dyn ProgramModuleRepository>,
    pub program_field_value_repository: Arc<dyn ProgramFieldValueRepository>,
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
            program_field_value_repository: Arc::new(ProgramFieldValueRepositoryPostgresImpl::new(postgres_connection)),
        }
    }
}
