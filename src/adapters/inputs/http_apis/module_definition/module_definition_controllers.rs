use crate::adapters::inputs::http_apis::module_definition::module_definition_mappers::{
    GetModuleDefinitionMapper, GetModuleDefinitionTreeMapper, ListModuleDefinitionsMapper,
};
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::module_definition::{
    GetModuleDefinitionPayload, ListModuleDefinitionsPayload,
};
use actix_web::{HttpResponse, get, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_module_definitions).service(get_module_definition);
}

#[instrument(skip(data))]
#[get("")]
pub async fn list_module_definitions(
    data: web::Data<AppState>,
    query: web::Query<ListModuleDefinitionsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListModuleDefinitionsController called");

    let query = query.into_inner();

    // `full_tree=true` swaps the paginated flat list for the nested tree.
    if query.full_tree.unwrap_or(false) {
        let tree_input = GetModuleDefinitionTreeMapper::input(query)
            .map_err(ErrorResponse::map_response_error)?;

        let result = data
            .use_cases
            .module_definition
            .get_module_definition_tree
            .execute(Some(tree_input))
            .await
            .map_err(ErrorResponse::map_response_error)?;

        return Ok(HttpResponse::Ok().json(GetModuleDefinitionTreeMapper::output_success(result)));
    }

    let usecase_input = ListModuleDefinitionsMapper::input(query)
        .map_err(ErrorResponse::map_response_error)?;

    let result = data
        .use_cases
        .module_definition
        .list_module_definitions
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListModuleDefinitionsMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{module_definition}")]
pub async fn get_module_definition(
    data: web::Data<AppState>,
    path: web::Path<GetModuleDefinitionPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetModuleDefinitionController called");

    let usecase_input = GetModuleDefinitionMapper::input(path.into_inner())
        .map_err(ErrorResponse::map_response_error)?;

    let result = data
        .use_cases
        .module_definition
        .get_module_definition
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetModuleDefinitionMapper::output_success(result)))
}
