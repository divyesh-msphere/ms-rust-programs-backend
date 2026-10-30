use crate::domain::document::DocumentPolicy;
use crate::domain::library_catalog::LibraryCatalog;
use crate::domain::object_storage::ObjectStorage;
use crate::infrastructure::repositories::Repositories;
use std::sync::Arc;

pub mod application;
pub mod approval_step;
pub mod audit_log;
pub mod document;
pub mod interface;
pub mod enrollment;
pub mod module_catalog_item;
pub mod program;
pub mod program_setting_value;
pub mod program_module;
pub mod danger_zone;
pub mod notifications;
pub mod program_overview;
pub mod signoff;
pub mod store_discount;
pub mod publication;
pub mod requirement;
pub mod rule_set;

pub struct UseCases {
    pub module_catalog_item: module_catalog_item::ModuleCatalogItemUseCases,
    pub program: program::ProgramUseCases,
    pub program_module: program_module::ProgramModuleUseCases,
    pub program_setting_value: program_setting_value::ProgramSettingValueUseCases,
    pub program_overview: program_overview::ProgramOverviewUseCases,
    pub danger_zone: danger_zone::DangerZoneUseCases,
    pub notifications: notifications::NotificationsUseCases,
    pub audit_log: audit_log::AuditLogUseCases,
    pub requirement: requirement::RequirementUseCases,
    pub approval_step: approval_step::ApprovalStepUseCases,
    pub application: application::ApplicationUseCases,
    pub signoff: signoff::SignoffUseCases,
    pub enrollment: enrollment::EnrollmentUseCases,
    pub store_discount: store_discount::StoreDiscountUseCases,
    pub rule_set: rule_set::RuleSetUseCases,
    pub publication: publication::PublicationUseCases,
    pub document: document::DocumentUseCases,
}

impl UseCases {
    pub fn new(
        repositories: Arc<Repositories>,
        library_catalog: Arc<dyn LibraryCatalog>,
        object_storage: Arc<dyn ObjectStorage>,
        document_policy: DocumentPolicy,
    ) -> Self {
        Self {
            module_catalog_item: module_catalog_item::ModuleCatalogItemUseCases::new(
                repositories.module_catalog_item_repository.clone(),
            ),
            program: program::ProgramUseCases::new(repositories.program_repository.clone(), library_catalog.clone()),
            program_module: program_module::ProgramModuleUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_module_repository.clone(),
                library_catalog.clone(),
            ),
            program_setting_value: program_setting_value::ProgramSettingValueUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_setting_value_repository.clone(),
                library_catalog,
            ),
            program_overview: program_overview::ProgramOverviewUseCases::new(
                repositories.program_repository.clone(),
                repositories.program_overview_repository.clone(),
            ),
            danger_zone: danger_zone::DangerZoneUseCases::new(
                repositories.program_repository.clone(),
                repositories.danger_zone_repository.clone(),
            ),
            notifications: notifications::NotificationsUseCases::new(
                repositories.program_repository.clone(),
                repositories.notifications_repository.clone(),
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
            signoff: signoff::SignoffUseCases::new(
                repositories.program_repository.clone(),
                repositories.signoff_repository.clone(),
            ),
            enrollment: enrollment::EnrollmentUseCases::new(
                repositories.program_repository.clone(),
                repositories.enrollment_repository.clone(),
            ),
            store_discount: store_discount::StoreDiscountUseCases::new(
                repositories.program_repository.clone(),
                repositories.store_discount_repository.clone(),
            ),
            rule_set: rule_set::RuleSetUseCases::new(
                repositories.program_repository.clone(),
                repositories.rule_set_repository.clone(),
                repositories.rule_engine.clone(),
            ),
            publication: publication::PublicationUseCases::new(
                repositories.program_repository.clone(),
                repositories.publication_repository.clone(),
            ),
            document: document::DocumentUseCases::new(
                repositories.program_repository.clone(),
                repositories.document_repository.clone(),
                object_storage,
                document_policy,
            ),
        }
    }
}
