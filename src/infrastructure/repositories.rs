use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::config::ApplicationConfig;
use std::sync::Arc;

pub struct Repositories {}

impl Repositories {
    pub async fn new(app_config: Arc<ApplicationConfig>) -> Self {
        let _postgres_connection = Arc::new(PostgresDbConnection::new(&app_config.postgres));

        Self {}
    }
}
