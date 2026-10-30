use crate::adapters::generated::grpc_services::buffa::programs::v1::*;
use crate::adapters::generated::grpc_services::connect::programs::v1::ProgramsService;
use crate::adapters::inputs::grpc_apis::application::application_service_impl::ApplicationServiceImpl;
use crate::adapters::inputs::grpc_apis::approval_step::approval_step_service_impl::ApprovalStepServiceImpl;
use crate::adapters::inputs::grpc_apis::danger_zone::danger_zone_service_impl::DangerZoneServiceImpl;
use crate::adapters::inputs::grpc_apis::document::document_service_impl::DocumentServiceImpl;
use crate::adapters::inputs::grpc_apis::enrollment::enrollment_service_impl::EnrollmentServiceImpl;
use crate::adapters::inputs::grpc_apis::notifications::notifications_service_impl::NotificationsServiceImpl;
use crate::adapters::inputs::grpc_apis::health::health_service_impl::HealthServiceImpl;
use crate::adapters::inputs::grpc_apis::module_catalog_item::module_catalog_item_service_impl::ModuleCatalogItemServiceImpl;
use crate::adapters::inputs::grpc_apis::program::program_service_impl::ProgramServiceImpl;
use crate::adapters::inputs::grpc_apis::program_setting_value::program_setting_value_service_impl::ProgramSettingValueServiceImpl;
use crate::adapters::inputs::grpc_apis::program_module::program_module_service_impl::ProgramModuleServiceImpl;
use crate::adapters::inputs::grpc_apis::program_overview::program_overview_service_impl::ProgramOverviewServiceImpl;
use crate::adapters::inputs::grpc_apis::requirement::requirement_service_impl::RequirementServiceImpl;
use crate::adapters::inputs::grpc_apis::signoff::signoff_service_impl::SignoffServiceImpl;
use crate::adapters::inputs::grpc_apis::store_discount::store_discount_service_impl::StoreDiscountServiceImpl;
use crate::adapters::inputs::grpc_apis::publication::publication_service_impl::PublicationServiceImpl;
use crate::adapters::inputs::grpc_apis::rule_set::rule_set_service_impl::RuleSetServiceImpl;
use crate::adapters::inputs::grpc_apis::shared::actor::actor_from;
use crate::adapters::inputs::shared::app_state::AppState;
use buffa_types::google::protobuf::Empty;
use connectrpc::{ConnectError, ErrorCode, RequestContext, ServiceRequest, ServiceResult};
use std::sync::Arc;

/// One gRPC service (`ProgramsService`, mirroring the single-service shape of
/// `programs.proto`) fanning out to the per-resource impls below — each keeps
/// its own file, mappers and business logic; this struct only routes.
pub struct ProgramsServiceImpl {
    module_catalog_item: ModuleCatalogItemServiceImpl,
    program: ProgramServiceImpl,
    program_module: ProgramModuleServiceImpl,
    program_setting_value: ProgramSettingValueServiceImpl,
    program_overview: ProgramOverviewServiceImpl,
    danger_zone: DangerZoneServiceImpl,
    notifications: NotificationsServiceImpl,
    requirement: RequirementServiceImpl,
    approval_step: ApprovalStepServiceImpl,
    application: ApplicationServiceImpl,
    signoff: SignoffServiceImpl,
    enrollment: EnrollmentServiceImpl,
    store_discount: StoreDiscountServiceImpl,
    rule_set: RuleSetServiceImpl,
    publication: PublicationServiceImpl,
    document: DocumentServiceImpl,
    health: HealthServiceImpl,
    /// `[features] enrollment_review`: Requirements, Approval steps, Applications and Signoffs.
    enrollment_review: bool,
}

impl ProgramsServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            enrollment_review: app_state.application_config.features.enrollment_review,
            module_catalog_item: ModuleCatalogItemServiceImpl::new(app_state.clone()),
            program: ProgramServiceImpl::new(app_state.clone()),
            program_module: ProgramModuleServiceImpl::new(app_state.clone()),
            program_setting_value: ProgramSettingValueServiceImpl::new(app_state.clone()),
            program_overview: ProgramOverviewServiceImpl::new(app_state.clone()),
            danger_zone: DangerZoneServiceImpl::new(app_state.clone()),
            notifications: NotificationsServiceImpl::new(app_state.clone()),
            requirement: RequirementServiceImpl::new(app_state.clone()),
            approval_step: ApprovalStepServiceImpl::new(app_state.clone()),
            application: ApplicationServiceImpl::new(app_state.clone()),
            signoff: SignoffServiceImpl::new(app_state.clone()),
            enrollment: EnrollmentServiceImpl::new(app_state.clone()),
            store_discount: StoreDiscountServiceImpl::new(app_state.clone()),
            rule_set: RuleSetServiceImpl::new(app_state.clone()),
            publication: PublicationServiceImpl::new(app_state.clone()),
            document: DocumentServiceImpl::new(app_state.clone()),
            health: HealthServiceImpl::new(app_state),
        }
    }
}

impl ProgramsServiceImpl {
    /// Requirements, Approval steps, Applications and Signoffs are switched off by default
    /// (`[features] enrollment_review`); their rpcs stay in the service but answer UNIMPLEMENTED.
    fn require_enrollment_review(&self) -> Result<(), ConnectError> {
        if self.enrollment_review {
            Ok(())
        } else {
            Err(ConnectError::new(ErrorCode::Unimplemented, "this module is switched off (features.enrollment_review)"))
        }
    }
}

impl ProgramsService for ProgramsServiceImpl {
    // ---- Module catalog ----
    async fn list_module_catalog(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListModuleCatalogRequest>) -> ServiceResult<ListModuleCatalogResponse> {
        self.module_catalog_item.list_module_catalog(request).await
    }
    async fn get_module_catalog_item(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetModuleCatalogItemRequest>) -> ServiceResult<GetModuleCatalogItemResponse> {
        self.module_catalog_item.get_module_catalog_item(request).await
    }

    // ---- Programs ----
    async fn list_programs(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListProgramsRequest>) -> ServiceResult<ListProgramsResponse> {
        self.program.list_programs(request).await
    }
    async fn create_program(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateProgramRequest>) -> ServiceResult<CreateProgramResponse> {
        self.program.create_program(actor_from(&ctx), request).await
    }
    async fn get_program(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetProgramRequest>) -> ServiceResult<GetProgramResponse> {
        self.program.get_program(request).await
    }
    async fn update_program(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateProgramRequest>) -> ServiceResult<UpdateProgramResponse> {
        self.program.update_program(actor_from(&ctx), request).await
    }
    async fn delete_program(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteProgramRequest>) -> ServiceResult<Empty> {
        self.program.delete_program(actor_from(&ctx), request).await
    }
    async fn undelete_program(&self, ctx: RequestContext, request: ServiceRequest<'_, UndeleteProgramRequest>) -> ServiceResult<UndeleteProgramResponse> {
        self.program.undelete_program(actor_from(&ctx), request).await
    }

    // ---- Program modules ----
    async fn list_program_modules(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListProgramModulesRequest>) -> ServiceResult<ListProgramModulesResponse> {
        self.program_module.list_program_modules(request).await
    }

    // ---- Setting values ----
    async fn list_setting_values(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListSettingValuesRequest>) -> ServiceResult<ListSettingValuesResponse> {
        self.program_setting_value.list_setting_values(request).await
    }
    async fn batch_update_setting_values(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateSettingValuesRequest>) -> ServiceResult<BatchUpdateSettingValuesResponse> {
        self.program_setting_value.batch_update_setting_values(actor_from(&ctx), request).await
    }

    // ---- Requirements ----
    async fn list_requirements(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListRequirementsRequest>) -> ServiceResult<ListRequirementsResponse> {
        self.require_enrollment_review()?;
        self.requirement.list_requirements(request).await
    }
    async fn get_requirement(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetRequirementRequest>) -> ServiceResult<GetRequirementResponse> {
        self.require_enrollment_review()?;
        self.requirement.get_requirement(request).await
    }
    async fn create_requirement(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateRequirementRequest>) -> ServiceResult<CreateRequirementResponse> {
        self.require_enrollment_review()?;
        self.requirement.create_requirement(actor_from(&ctx), request).await
    }
    async fn update_requirement(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateRequirementRequest>) -> ServiceResult<UpdateRequirementResponse> {
        self.require_enrollment_review()?;
        self.requirement.update_requirement(actor_from(&ctx), request).await
    }
    async fn delete_requirement(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteRequirementRequest>) -> ServiceResult<Empty> {
        self.require_enrollment_review()?;
        self.requirement.delete_requirement(actor_from(&ctx), request).await
    }
    async fn batch_update_requirements(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateRequirementsRequest>) -> ServiceResult<BatchUpdateRequirementsResponse> {
        self.require_enrollment_review()?;
        self.requirement.batch_update_requirements(actor_from(&ctx), request).await
    }

    // ---- Approval steps ----
    async fn list_approval_steps(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListApprovalStepsRequest>) -> ServiceResult<ListApprovalStepsResponse> {
        self.require_enrollment_review()?;
        self.approval_step.list_approval_steps(request).await
    }
    async fn get_approval_step(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetApprovalStepRequest>) -> ServiceResult<GetApprovalStepResponse> {
        self.require_enrollment_review()?;
        self.approval_step.get_approval_step(request).await
    }
    async fn create_approval_step(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateApprovalStepRequest>) -> ServiceResult<CreateApprovalStepResponse> {
        self.require_enrollment_review()?;
        self.approval_step.create_approval_step(actor_from(&ctx), request).await
    }
    async fn update_approval_step(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateApprovalStepRequest>) -> ServiceResult<UpdateApprovalStepResponse> {
        self.require_enrollment_review()?;
        self.approval_step.update_approval_step(actor_from(&ctx), request).await
    }
    async fn delete_approval_step(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteApprovalStepRequest>) -> ServiceResult<Empty> {
        self.require_enrollment_review()?;
        self.approval_step.delete_approval_step(actor_from(&ctx), request).await
    }
    async fn batch_update_approval_steps(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateApprovalStepsRequest>) -> ServiceResult<BatchUpdateApprovalStepsResponse> {
        self.require_enrollment_review()?;
        self.approval_step.batch_update_approval_steps(actor_from(&ctx), request).await
    }

    // ---- Applications ----
    async fn list_applications(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListApplicationsRequest>) -> ServiceResult<ListApplicationsResponse> {
        self.require_enrollment_review()?;
        self.application.list_applications(request).await
    }
    async fn get_application(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetApplicationRequest>) -> ServiceResult<GetApplicationResponse> {
        self.require_enrollment_review()?;
        self.application.get_application(request).await
    }
    async fn create_application(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateApplicationRequest>) -> ServiceResult<CreateApplicationResponse> {
        self.require_enrollment_review()?;
        self.application.create_application(actor_from(&ctx), request).await
    }
    async fn submit_application(&self, ctx: RequestContext, request: ServiceRequest<'_, SubmitApplicationRequest>) -> ServiceResult<SubmitApplicationResponse> {
        self.require_enrollment_review()?;
        self.application.submit_application(actor_from(&ctx), request).await
    }
    async fn withdraw_application(&self, ctx: RequestContext, request: ServiceRequest<'_, WithdrawApplicationRequest>) -> ServiceResult<WithdrawApplicationResponse> {
        self.require_enrollment_review()?;
        self.application.withdraw_application(actor_from(&ctx), request).await
    }

    // ---- Signoffs ----
    async fn list_signoffs(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListSignoffsRequest>) -> ServiceResult<ListSignoffsResponse> {
        self.require_enrollment_review()?;
        self.signoff.list_signoffs(request).await
    }
    async fn get_signoff(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetSignoffRequest>) -> ServiceResult<GetSignoffResponse> {
        self.require_enrollment_review()?;
        self.signoff.get_signoff(request).await
    }
    async fn create_signoff(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateSignoffRequest>) -> ServiceResult<CreateSignoffResponse> {
        self.require_enrollment_review()?;
        self.signoff.create_signoff(actor_from(&ctx), request).await
    }

    // ---- Enrollments ----
    async fn list_enrollments(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListEnrollmentsRequest>) -> ServiceResult<ListEnrollmentsResponse> {
        self.enrollment.list_enrollments(request).await
    }
    async fn get_enrollment(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetEnrollmentRequest>) -> ServiceResult<GetEnrollmentResponse> {
        self.enrollment.get_enrollment(request).await
    }
    async fn create_enrollment(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateEnrollmentRequest>) -> ServiceResult<CreateEnrollmentResponse> {
        self.enrollment.create_enrollment(actor_from(&ctx), request).await
    }
    async fn batch_create_enrollments(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchCreateEnrollmentsRequest>) -> ServiceResult<BatchCreateEnrollmentsResponse> {
        self.enrollment.batch_create_enrollments(actor_from(&ctx), request).await
    }
    async fn update_enrollment(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateEnrollmentRequest>) -> ServiceResult<UpdateEnrollmentResponse> {
        self.enrollment.update_enrollment(actor_from(&ctx), request).await
    }

    // ---- Store discounts ----
    async fn list_store_discounts(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListStoreDiscountsRequest>) -> ServiceResult<ListStoreDiscountsResponse> {
        self.store_discount.list_store_discounts(request).await
    }
    async fn get_store_discount(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetStoreDiscountRequest>) -> ServiceResult<GetStoreDiscountResponse> {
        self.store_discount.get_store_discount(request).await
    }
    async fn update_store_discount(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateStoreDiscountRequest>) -> ServiceResult<UpdateStoreDiscountResponse> {
        self.store_discount.update_store_discount(actor_from(&ctx), request).await
    }
    async fn batch_update_store_discounts(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateStoreDiscountsRequest>) -> ServiceResult<BatchUpdateStoreDiscountsResponse> {
        self.store_discount.batch_update_store_discounts(actor_from(&ctx), request).await
    }
    async fn list_discount_levels(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListDiscountLevelsRequest>) -> ServiceResult<ListDiscountLevelsResponse> {
        self.store_discount.list_discount_levels(request).await
    }
    async fn update_discount_level(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateDiscountLevelRequest>) -> ServiceResult<UpdateDiscountLevelResponse> {
        self.store_discount.update_discount_level(actor_from(&ctx), request).await
    }

    // ---- Rule sets and publishing ----
    async fn list_rule_sets(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListRuleSetsRequest>) -> ServiceResult<ListRuleSetsResponse> {
        self.rule_set.list_rule_sets(request).await
    }
    async fn get_rule_set(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetRuleSetRequest>) -> ServiceResult<GetRuleSetResponse> {
        self.rule_set.get_rule_set(request).await
    }
    async fn create_rule_version(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateRuleVersionRequest>) -> ServiceResult<CreateRuleVersionResponse> {
        self.rule_set.create_rule_version(actor_from(&ctx), request).await
    }
    async fn execute_rule(&self, _ctx: RequestContext, request: ServiceRequest<'_, ExecuteRuleRequest>) -> ServiceResult<ExecuteRuleResponse> {
        self.rule_set.execute_rule(request).await
    }
    async fn list_rule_versions(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListRuleVersionsRequest>) -> ServiceResult<ListRuleVersionsResponse> {
        self.rule_set.list_rule_versions(request).await
    }
    async fn get_rule_version(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetRuleVersionRequest>) -> ServiceResult<GetRuleVersionResponse> {
        self.rule_set.get_rule_version(request).await
    }
    async fn list_publications(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListPublicationsRequest>) -> ServiceResult<ListPublicationsResponse> {
        self.publication.list_publications(request).await
    }
    async fn create_publication(&self, ctx: RequestContext, request: ServiceRequest<'_, CreatePublicationRequest>) -> ServiceResult<CreatePublicationResponse> {
        self.publication.create_publication(actor_from(&ctx), request).await
    }
    async fn get_publication(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetPublicationRequest>) -> ServiceResult<GetPublicationResponse> {
        self.publication.get_publication(request).await
    }

    // ---- Documents ----
    async fn get_document_summary(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetDocumentSummaryRequest>) -> ServiceResult<GetDocumentSummaryResponse> {
        self.document.get_document_summary(request).await
    }
    async fn list_document_folders(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListDocumentFoldersRequest>) -> ServiceResult<ListDocumentFoldersResponse> {
        self.document.list_document_folders(request).await
    }
    async fn create_document_folder(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateDocumentFolderRequest>) -> ServiceResult<CreateDocumentFolderResponse> {
        self.document.create_document_folder(actor_from(&ctx), request).await
    }
    async fn update_document_folder(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateDocumentFolderRequest>) -> ServiceResult<UpdateDocumentFolderResponse> {
        self.document.update_document_folder(actor_from(&ctx), request).await
    }
    async fn delete_document_folder(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteDocumentFolderRequest>) -> ServiceResult<Empty> {
        self.document.delete_document_folder(actor_from(&ctx), request).await
    }
    async fn list_documents(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListDocumentsRequest>) -> ServiceResult<ListDocumentsResponse> {
        self.document.list_documents(request).await
    }
    async fn create_document(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateDocumentRequest>) -> ServiceResult<CreateDocumentResponse> {
        self.document.create_document(actor_from(&ctx), request).await
    }
    async fn complete_document(&self, ctx: RequestContext, request: ServiceRequest<'_, CompleteDocumentRequest>) -> ServiceResult<CompleteDocumentResponse> {
        self.document.complete_document(actor_from(&ctx), request).await
    }
    async fn get_document(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetDocumentRequest>) -> ServiceResult<GetDocumentResponse> {
        self.document.get_document(request).await
    }
    async fn update_document(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateDocumentRequest>) -> ServiceResult<UpdateDocumentResponse> {
        self.document.update_document(actor_from(&ctx), request).await
    }
    async fn delete_document(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteDocumentRequest>) -> ServiceResult<Empty> {
        self.document.delete_document(actor_from(&ctx), request).await
    }
    async fn download_document(&self, _ctx: RequestContext, request: ServiceRequest<'_, DownloadDocumentRequest>) -> ServiceResult<DownloadDocumentResponse> {
        self.document.download_document(request).await
    }
    async fn review_document(&self, ctx: RequestContext, request: ServiceRequest<'_, ReviewDocumentRequest>) -> ServiceResult<ReviewDocumentResponse> {
        self.document.review_document(actor_from(&ctx), request).await
    }

    // ---- Danger zone ----
    async fn get_danger_zone(&self, ctx: RequestContext, request: ServiceRequest<'_, GetDangerZoneRequest>) -> ServiceResult<GetDangerZoneResponse> {
        self.danger_zone.get_danger_zone(actor_from(&ctx), request).await
    }
    async fn archive_program(&self, ctx: RequestContext, request: ServiceRequest<'_, ArchiveProgramRequest>) -> ServiceResult<ArchiveProgramResponse> {
        self.danger_zone.archive_program(actor_from(&ctx), request).await
    }
    async fn unarchive_program(&self, ctx: RequestContext, request: ServiceRequest<'_, UnarchiveProgramRequest>) -> ServiceResult<UnarchiveProgramResponse> {
        self.danger_zone.unarchive_program(actor_from(&ctx), request).await
    }
    async fn delete_program_permanently(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteProgramPermanentlyRequest>) -> ServiceResult<Empty> {
        self.danger_zone.delete_program_permanently(actor_from(&ctx), request).await
    }

    // ---- Notifications ----
    async fn get_notifications(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetNotificationsRequest>) -> ServiceResult<GetNotificationsResponse> {
        self.notifications.get_notifications(request).await
    }
    async fn replace_notifications(&self, ctx: RequestContext, request: ServiceRequest<'_, ReplaceNotificationsRequest>) -> ServiceResult<ReplaceNotificationsResponse> {
        self.notifications.replace_notifications(actor_from(&ctx), request).await
    }

    // ---- Overview and history ----
    async fn get_overview(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetOverviewRequest>) -> ServiceResult<GetOverviewResponse> {
        self.program_overview.get_overview(request).await
    }
    async fn list_audit_logs(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListAuditLogsRequest>) -> ServiceResult<ListAuditLogsResponse> {
        self.program_overview.list_audit_logs(request).await
    }

    // ---- Health ----
    async fn list_health_check(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListHealthCheckRequest>) -> ServiceResult<ListHealthCheckResponse> {
        self.health.list_health_check(request).await
    }
}
