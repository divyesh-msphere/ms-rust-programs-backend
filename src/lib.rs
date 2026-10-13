use crate::application::usecases::UseCases;
use crate::config::ApplicationConfig;
use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

pub mod adapters;
pub mod application;
pub mod config;
pub mod domain;
pub mod dto;
pub mod infrastructure;

pub struct Dependencies {
    pub application_config: Arc<ApplicationConfig>,
    pub repositories: Arc<Repositories>,
    pub use_cases: Arc<UseCases>,
}

impl Dependencies {
    pub async fn load() -> Self {
        let app_cfg =
            Arc::new(ApplicationConfig::new().expect("Failed to load application configuration"));
        let repositories = Arc::new(Repositories::new(app_cfg.clone()).await);
        let use_cases = Arc::new(UseCases::new(repositories.clone()));

        Self {
            application_config: app_cfg,
            repositories,
            use_cases,
        }
    }
}
