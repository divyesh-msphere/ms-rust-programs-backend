use crate::domain::library_catalog::LibraryCatalog;
use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

pub mod application;
pub mod approval_step;
pub mod audit_log;
pub mod interface;
pub mod module_definition;
pub mod program;
pub mod program_field_value;
pub mod program_module;
pub mod program_overview;
pub mod requirement;

pub struct UseCases {
    pub module_definition: module_definition::ModuleDefinitionUseCases,
    pub program: program::ProgramUseCases,
    pub program_module: program_module::ProgramModuleUseCases,
    pub program_field_value: program_field_value::ProgramFieldValueUseCases,
    pub program_overview: program_overview::ProgramOverviewUseCases,
    pub audit_log: audit_log::AuditLogUseCases,
    pub requirement: requirement::RequirementUseCases,
    pub approval_step: approval_step::ApprovalStepUseCases,
    pub application: application::ApplicationUseCases,
}

impl UseCases {
    pub fn new(repositories: Arc<Repositories>, library_catalog: Arc<dyn LibraryCatalog>) -> Self {
        Self {
            module_definition: module_definition::ModuleDefinitionUseCases::new(
                repositories.module_definition_repository.clone(),
            ),
            program: program::ProgramUseCases::new(repositories.program_repository.clone(), library_catalog.clone()),
            program_module: program_module::ProgramModuleUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_module_repository.clone(),
                library_catalog.clone(),
            ),
            program_field_value: program_field_value::ProgramFieldValueUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_field_value_repository.clone(),
                library_catalog,
            ),
            program_overview: program_overview::ProgramOverviewUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_overview_repository.clone(),
            ),
            audit_log: audit_log::AuditLogUseCases::new(
                repositories.program_repository.clone(),
                repositories.audit_log_repository.clone(),
            ),
            requirement: requirement::RequirementUseCases::new(
                repositories.program_repository.clone(),
                repositories.requirement_repository.clone(),
            ),
            approval_step: approval_step::ApprovalStepUseCases::new(
                repositories.program_repository.clone(),
                repositories.approval_step_repository.clone(),
            ),
            application: application::ApplicationUseCases::new(
                repositories.program_repository.clone(),
                repositories.application_repository.clone(),
            ),
        }
    }
}
