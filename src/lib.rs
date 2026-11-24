use crate::adapters::services::object_storage::s3_client::S3Client;
use crate::application::usecases::UseCases;
use crate::config::ApplicationConfig;
use crate::domain::document::DocumentPolicy;
use crate::domain::object_storage::ObjectStorage;
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
        // Document bytes live in object storage; this service only signs the URLs and checks what arrived.
        let object_storage: Arc<dyn ObjectStorage> = Arc::new(S3Client::new(&app_cfg.storage));
        let document_policy = DocumentPolicy {
            max_file_bytes: app_cfg.storage.max_file_bytes,
            allowed_content_types: app_cfg.storage.allowed_content_types.clone(),
            max_files_per_program: app_cfg.storage.max_files_per_program,
        };
        let use_cases = Arc::new(UseCases::new(repositories.clone(), object_storage, document_policy));

        Self {
            application_config: app_cfg,
            repositories,
            use_cases,
        }
    }
}
