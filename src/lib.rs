use crate::adapters::services::global_settings::global_settings_client::GlobalSettingsClient;
use crate::application::usecases::UseCases;
use crate::config::ApplicationConfig;
use crate::domain::library_catalog::LibraryCatalog;
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
    /// Where library-backed settings read their options (global-settings).
    pub library_catalog: Arc<dyn LibraryCatalog>,
}

impl Dependencies {
    pub async fn load() -> Self {
        let app_cfg =
            Arc::new(ApplicationConfig::new().expect("Failed to load application configuration"));
        let repositories = Arc::new(Repositories::new(app_cfg.clone()).await);
        let library_catalog: Arc<dyn LibraryCatalog> =
            Arc::new(GlobalSettingsClient::new(&app_cfg.global_settings));
        let use_cases = Arc::new(UseCases::new(repositories.clone(), library_catalog.clone()));

        // Warm the library cache in the background, so even the first page load after a start
        // reads library options from memory. A failure only means a first read costs one call.
        if library_catalog.enabled() {
            let repositories = repositories.clone();
            let catalog = library_catalog.clone();
            tokio::spawn(async move {
                let Ok(ids) = repositories.program_module_repository.library_ids_in_use().await else { return };
                for id in ids {
                    let _ = catalog.get(id).await;
                }
            });
        }

        Self {
            application_config: app_cfg,
            repositories,
            use_cases,
            library_catalog,
        }
    }
}
