use crate::adapters::generated::grpc_services::buffa::programs::v1::*;
use crate::adapters::generated::grpc_services::connect::programs::v1::ProgramsService;
use crate::adapters::inputs::grpc_apis::health::health_service_impl::HealthServiceImpl;
use crate::adapters::inputs::grpc_apis::module_definition::module_definition_service_impl::ModuleDefinitionServiceImpl;
use crate::adapters::inputs::shared::app_state::AppState;
use connectrpc::{RequestContext, ServiceRequest, ServiceResult};
use std::sync::Arc;

/// One gRPC service (`ProgramsService`, mirroring the single-service shape of
/// `programs.proto`) fanning out to the per-resource impls below — each keeps
/// its own file, mappers and business logic; this struct only routes.
pub struct ProgramsServiceImpl {
    module_definition: ModuleDefinitionServiceImpl,
    health: HealthServiceImpl,
}

impl ProgramsServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            module_definition: ModuleDefinitionServiceImpl::new(app_state.clone()),
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

    // ---- Health ----
    async fn list_health_check(&self, _ctx: RequestContext, request: ServiceRequest<'_, ListHealthCheckRequest>) -> ServiceResult<ListHealthCheckResponse> {
        self.health.list_health_check(request).await
    }
}
