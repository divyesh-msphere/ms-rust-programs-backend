use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::module_definition::module_definition_repository::ModuleDefinitionRepositoryPostgresImpl;
use crate::config::ApplicationConfig;
use crate::domain::module_definition::ModuleDefinitionRepository;
use std::sync::Arc;

pub struct Repositories {
    pub module_definition_repository: Arc<dyn ModuleDefinitionRepository>,
}

impl Repositories {
    pub async fn new(app_config: Arc<ApplicationConfig>) -> Self {
        let postgres_connection = Arc::new(PostgresDbConnection::new(&app_config.postgres));

        Self {
            module_definition_repository: Arc::new(ModuleDefinitionRepositoryPostgresImpl::new(
                postgres_connection.clone(),
            )),
        }
    }
}
