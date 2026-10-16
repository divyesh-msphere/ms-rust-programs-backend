use crate::adapters::generated::grpc_services::buffa::programs::v1::*;
use crate::adapters::generated::grpc_services::connect::programs::v1::ProgramsService;
use crate::adapters::inputs::grpc_apis::health::health_service_impl::HealthServiceImpl;
use crate::adapters::inputs::grpc_apis::module_definition::module_definition_service_impl::ModuleDefinitionServiceImpl;
use crate::adapters::inputs::grpc_apis::program::program_service_impl::ProgramServiceImpl;
use crate::adapters::inputs::grpc_apis::program_field_value::program_field_value_service_impl::ProgramFieldValueServiceImpl;
use crate::adapters::inputs::grpc_apis::program_module::program_module_service_impl::ProgramModuleServiceImpl;
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
    health: HealthServiceImpl,
}

impl ProgramsServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            module_definition: ModuleDefinitionServiceImpl::new(app_state.clone()),
            program: ProgramServiceImpl::new(app_state.clone()),
            program_module: ProgramModuleServiceImpl::new(app_state.clone()),
            program_field_value: ProgramFieldValueServiceImpl::new(app_state.clone()),
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

    // ---- Health ----
    async fn list_health_check(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListHealthCheckRequest>) -> ServiceResult<ListHealthCheckResponse> {
        self.health.list_health_check(request).await
    }
}
