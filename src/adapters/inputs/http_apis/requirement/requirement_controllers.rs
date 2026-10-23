use crate::adapters::inputs::http_apis::requirement::requirement_mappers::{
    BatchUpdateRequirementsMapper, CreateRequirementMapper, DeleteRequirementMapper, GetRequirementMapper,
    ListRequirementsMapper, UpdateRequirementMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::requirement::{
    BatchUpdateRequirementsBodyPayload, BatchUpdateRequirementsPathPayload, CreateRequirementBodyPayload,
    CreateRequirementPathPayload, CreateRequirementQueryPayload, DeleteRequirementPayload, GetRequirementPayload,
    ListRequirementsPayload, UpdateRequirementBodyPayload, UpdateRequirementPathPayload,
};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_requirements)
        .service(create_requirement)
        .service(batch_update_requirements)
        .service(get_requirement)
        .service(update_requirement)
        .service(delete_requirement);
}

/// A program's requirements in display order. There are few of them, so the list is not paged.
#[instrument(skip(data))]
#[get("/{program}/requirements")]
pub async fn list_requirements(
    data: web::Data<AppState>,
    path: web::Path<ListRequirementsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListRequirementsController called");

    let usecase_input = ListRequirementsMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .requirement
        .list_requirements
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListRequirementsMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[post("/{program}/requirements")]
pub async fn create_requirement(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreateRequirementPathPayload>,
    query: web::Query<CreateRequirementQueryPayload>,
    body: web::Json<CreateRequirementBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateRequirementController called");

    let usecase_input =
        CreateRequirementMapper::input(path.into_inner(), query.into_inner(), body.into_inner(), actor_from(&req))
            .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .requirement
        .create_requirement
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateRequirementMapper::output_success(result)))
}

/// Sets the order of many requirements in one request.
#[instrument(skip(data, req, body))]
#[post("/{program}/requirements:batchUpdate")]
pub async fn batch_update_requirements(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<BatchUpdateRequirementsPathPayload>,
    body: web::Json<BatchUpdateRequirementsBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("BatchUpdateRequirementsController called");

    let usecase_input = BatchUpdateRequirementsMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .requirement
        .batch_update_requirements
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(BatchUpdateRequirementsMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/requirements/{requirement}")]
pub async fn get_requirement(
    data: web::Data<AppState>,
    path: web::Path<GetRequirementPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetRequirementController called");

    let usecase_input = GetRequirementMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .requirement
        .get_requirement
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetRequirementMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[patch("/{program}/requirements/{requirement}")]
pub async fn update_requirement(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateRequirementPathPayload>,
    body: web::Json<UpdateRequirementBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateRequirementController called");

    let usecase_input = UpdateRequirementMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .requirement
        .update_requirement
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateRequirementMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[delete("/{program}/requirements/{requirement}")]
pub async fn delete_requirement(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DeleteRequirementPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("DeleteRequirementController called");

    let usecase_input = DeleteRequirementMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    data.use_cases
        .requirement
        .delete_requirement
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::NoContent().finish())
}
