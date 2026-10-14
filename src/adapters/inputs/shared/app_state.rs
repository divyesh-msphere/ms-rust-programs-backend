use crate::application::usecases::UseCases;
use crate::config::ApplicationConfig;
use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

/// Transport-agnostic application state, built from `Dependencies::load()` and
/// shared by the HTTP (`http_apis`) and gRPC (`grpc_apis`) adapters.
pub struct AppState {
    pub application_config: Arc<ApplicationConfig>,
    pub repositories: Arc<Repositories>,
    pub use_cases: Arc<UseCases>,
}

impl AppState {
    pub fn new(
        app_config: Arc<ApplicationConfig>,
        repos: Arc<Repositories>,
        use_cases: Arc<UseCases>,
    ) -> Self {
        Self {
            application_config: app_config,
            repositories: repos,
            use_cases,
        }
    }
}
