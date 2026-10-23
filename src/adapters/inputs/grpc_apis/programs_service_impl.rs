use crate::adapters::generated::grpc_services::buffa::programs::v1::*;
use crate::adapters::generated::grpc_services::connect::programs::v1::ProgramsService;
use crate::adapters::inputs::grpc_apis::application::application_service_impl::ApplicationServiceImpl;
use crate::adapters::inputs::grpc_apis::approval_step::approval_step_service_impl::ApprovalStepServiceImpl;
use crate::adapters::inputs::grpc_apis::health::health_service_impl::HealthServiceImpl;
use crate::adapters::inputs::grpc_apis::module_definition::module_definition_service_impl::ModuleDefinitionServiceImpl;
use crate::adapters::inputs::grpc_apis::program::program_service_impl::ProgramServiceImpl;
use crate::adapters::inputs::grpc_apis::program_field_value::program_field_value_service_impl::ProgramFieldValueServiceImpl;
use crate::adapters::inputs::grpc_apis::program_module::program_module_service_impl::ProgramModuleServiceImpl;
use crate::adapters::inputs::grpc_apis::program_overview::program_overview_service_impl::ProgramOverviewServiceImpl;
use crate::adapters::inputs::grpc_apis::requirement::requirement_service_impl::RequirementServiceImpl;
use crate::adapters::inputs::grpc_apis::shared::actor::actor_from;
use crate::adapters::inputs::shared::app_state::AppState;
use buffa_types::google::protobuf::Empty;
use connectrpc::{RequestContext, ServiceRequest, ServiceResult};
use std::sync::Arc;

/// One gRPC service (`ProgramsService`, mirroring the single-service shape of
/// `programs.proto`) fanning out to the per-resource impls below — each keeps
/// its own file, mappers and business logic; this struct only routes.
pub struct ProgramsServiceImpl {
    module_definition: ModuleDefinitionServiceImpl,
    program: ProgramServiceImpl,
    program_module: ProgramModuleServiceImpl,
    program_field_value: ProgramFieldValueServiceImpl,
    program_overview: ProgramOverviewServiceImpl,
    requirement: RequirementServiceImpl,
    approval_step: ApprovalStepServiceImpl,
    application: ApplicationServiceImpl,
    health: HealthServiceImpl,
}

impl ProgramsServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            module_definition: ModuleDefinitionServiceImpl::new(app_state.clone()),
            program: ProgramServiceImpl::new(app_state.clone()),
            program_module: ProgramModuleServiceImpl::new(app_state.clone()),
            program_field_value: ProgramFieldValueServiceImpl::new(app_state.clone()),
            program_overview: ProgramOverviewServiceImpl::new(app_state.clone()),
            requirement: RequirementServiceImpl::new(app_state.clone()),
            approval_step: ApprovalStepServiceImpl::new(app_state.clone()),
            application: ApplicationServiceImpl::new(app_state.clone()),
            health: HealthServiceImpl::new(app_state),
        }
    }
}

impl ProgramsService for ProgramsServiceImpl {
    // ---- Module definitions ----
    async fn list_module_definitions(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListModuleDefinitionsRequest>) -> ServiceResult<ListModuleDefinitionsResponse> {
        self.module_definition.list_module_definitions(request).await
    }
    async fn get_module_definition(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetModuleDefinitionRequest>) -> ServiceResult<GetModuleDefinitionResponse> {
        self.module_definition.get_module_definition(request).await
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
    async fn list_field_values(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListFieldValuesRequest>) -> ServiceResult<ListFieldValuesResponse> {
        self.program_field_value.list_field_values(request).await
    }
    async fn batch_update_field_values(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateFieldValuesRequest>) -> ServiceResult<BatchUpdateFieldValuesResponse> {
        self.program_field_value.batch_update_field_values(actor_from(&ctx), request).await
    }

    // ---- Requirements ----
    async fn list_requirements(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListRequirementsRequest>) -> ServiceResult<ListRequirementsResponse> {
        self.requirement.list_requirements(request).await
    }
    async fn get_requirement(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetRequirementRequest>) -> ServiceResult<GetRequirementResponse> {
        self.requirement.get_requirement(request).await
    }
    async fn create_requirement(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateRequirementRequest>) -> ServiceResult<CreateRequirementResponse> {
        self.requirement.create_requirement(actor_from(&ctx), request).await
    }
    async fn update_requirement(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateRequirementRequest>) -> ServiceResult<UpdateRequirementResponse> {
        self.requirement.update_requirement(actor_from(&ctx), request).await
    }
    async fn delete_requirement(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteRequirementRequest>) -> ServiceResult<Empty> {
        self.requirement.delete_requirement(actor_from(&ctx), request).await
    }
    async fn batch_update_requirements(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateRequirementsRequest>) -> ServiceResult<BatchUpdateRequirementsResponse> {
        self.requirement.batch_update_requirements(actor_from(&ctx), request).await
    }

    // ---- Approval steps ----
    async fn list_approval_steps(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListApprovalStepsRequest>) -> ServiceResult<ListApprovalStepsResponse> {
        self.approval_step.list_approval_steps(request).await
    }
    async fn get_approval_step(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetApprovalStepRequest>) -> ServiceResult<GetApprovalStepResponse> {
        self.approval_step.get_approval_step(request).await
    }
    async fn create_approval_step(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateApprovalStepRequest>) -> ServiceResult<CreateApprovalStepResponse> {
        self.approval_step.create_approval_step(actor_from(&ctx), request).await
    }
    async fn update_approval_step(&self, ctx: RequestContext, request: ServiceRequest<'_, UpdateApprovalStepRequest>) -> ServiceResult<UpdateApprovalStepResponse> {
        self.approval_step.update_approval_step(actor_from(&ctx), request).await
    }
    async fn delete_approval_step(&self, ctx: RequestContext, request: ServiceRequest<'_, DeleteApprovalStepRequest>) -> ServiceResult<Empty> {
        self.approval_step.delete_approval_step(actor_from(&ctx), request).await
    }
    async fn batch_update_approval_steps(&self, ctx: RequestContext, request: ServiceRequest<'_, BatchUpdateApprovalStepsRequest>) -> ServiceResult<BatchUpdateApprovalStepsResponse> {
        self.approval_step.batch_update_approval_steps(actor_from(&ctx), request).await
    }

    // ---- Applications ----
    async fn list_applications(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListApplicationsRequest>) -> ServiceResult<ListApplicationsResponse> {
        self.application.list_applications(request).await
    }
    async fn get_application(&self, _ctx: RequestContext, request: ServiceRequest<'_, GetApplicationRequest>) -> ServiceResult<GetApplicationResponse> {
        self.application.get_application(request).await
    }
    async fn create_application(&self, ctx: RequestContext, request: ServiceRequest<'_, CreateApplicationRequest>) -> ServiceResult<CreateApplicationResponse> {
        self.application.create_application(actor_from(&ctx), request).await
    }
    async fn submit_application(&self, ctx: RequestContext, request: ServiceRequest<'_, SubmitApplicationRequest>) -> ServiceResult<SubmitApplicationResponse> {
        self.application.submit_application(actor_from(&ctx), request).await
    }
    async fn withdraw_application(&self, ctx: RequestContext, request: ServiceRequest<'_, WithdrawApplicationRequest>) -> ServiceResult<WithdrawApplicationResponse> {
        self.application.withdraw_application(actor_from(&ctx), request).await
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
