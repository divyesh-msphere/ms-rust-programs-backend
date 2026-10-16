use crate::adapters::inputs::http_apis::program_module::program_module_mappers::ListProgramModulesMapper;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::program_module::{ListProgramModulesPathPayload, ListProgramModulesQueryPayload};
use actix_web::{HttpResponse, get, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_program_modules);
}

/// The program's module manifest: what to draw and which API each module uses.
#[instrument(skip(data))]
#[get("/{program}/modules")]
pub async fn list_program_modules(
    data: web::Data<AppState>,
    path: web::Path<ListProgramModulesPathPayload>,
    query: web::Query<ListProgramModulesQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListProgramModulesController called");

    let program = path.program.clone();
    let usecase_input = ListProgramModulesMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;

    let result = data
        .use_cases
        .program_module
        .list_program_modules
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListProgramModulesMapper::output_success(&format!("programs/{program}"), result)))
}
