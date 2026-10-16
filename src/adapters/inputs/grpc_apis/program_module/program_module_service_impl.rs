use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    ListProgramModulesRequest, ListProgramModulesResponse,
};
use crate::adapters::inputs::grpc_apis::program_module::program_module_mappers::ListProgramModulesGrpcMapper;
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ProgramModuleServiceImpl {
    app_state: Arc<AppState>,
}

impl ProgramModuleServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ProgramModuleServiceImpl {
    pub async fn list_program_modules(
        &self,
        request: ServiceRequest<'_, ListProgramModulesRequest>,
    ) -> ServiceResult<ListProgramModulesResponse> {
        let request = request.to_owned_message();
        let program = request.program.clone();

        let input =
            ListProgramModulesGrpcMapper::input(request).map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program_module
            .list_program_modules
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListProgramModulesGrpcMapper::output_with_path(&format!("programs/{program}"), output))
    }
}
