use crate::adapters::inputs::http_apis::module_catalog_item::module_catalog_item_mappers::{
    GetModuleCatalogItemMapper, GetModuleCatalogTreeMapper, ListModuleCatalogMapper,
};
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::module_catalog_item::{
    GetModuleCatalogItemPayload, ListModuleCatalogPayload,
};
use actix_web::{HttpResponse, get, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_module_catalog).service(get_module_catalog_item);
}

#[instrument(skip(data))]
#[get("")]
pub async fn list_module_catalog(
    data: web::Data<AppState>,
    query: web::Query<ListModuleCatalogPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListModuleCatalogController called");

    let query = query.into_inner();

    // `full_tree=true` swaps the paginated flat list for the nested tree.
    if query.full_tree.unwrap_or(false) {
        let tree_input = GetModuleCatalogTreeMapper::input(query)
            .map_err(ErrorResponse::map_response_error)?;

        let result = data
            .use_cases
            .module_catalog_item
            .get_module_catalog_tree
            .execute(Some(tree_input))
            .await
            .map_err(ErrorResponse::map_response_error)?;

        return Ok(HttpResponse::Ok().json(GetModuleCatalogTreeMapper::output_success(result)));
    }

    let usecase_input = ListModuleCatalogMapper::input(query)
        .map_err(ErrorResponse::map_response_error)?;

    let result = data
        .use_cases
        .module_catalog_item
        .list_module_catalog
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListModuleCatalogMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{module_catalog_item}")]
pub async fn get_module_catalog_item(
    data: web::Data<AppState>,
    path: web::Path<GetModuleCatalogItemPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetModuleCatalogItemController called");

    let usecase_input = GetModuleCatalogItemMapper::input(path.into_inner())
        .map_err(ErrorResponse::map_response_error)?;

    let result = data
        .use_cases
        .module_catalog_item
        .get_module_catalog_item
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetModuleCatalogItemMapper::output_success(result)))
}
