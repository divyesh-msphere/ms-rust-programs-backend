use crate::adapters::inputs::http_apis::approval_step::approval_step_mappers::{
    BatchUpdateApprovalStepsMapper, CreateApprovalStepMapper, DeleteApprovalStepMapper, GetApprovalStepMapper,
    ListApprovalStepsMapper, UpdateApprovalStepMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::approval_step::{
    BatchUpdateApprovalStepsBodyPayload, BatchUpdateApprovalStepsPathPayload, CreateApprovalStepBodyPayload,
    CreateApprovalStepPathPayload, CreateApprovalStepQueryPayload, DeleteApprovalStepPayload, GetApprovalStepPayload,
    ListApprovalStepsPayload, UpdateApprovalStepBodyPayload, UpdateApprovalStepPathPayload,
};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_approval_steps)
        .service(create_approval_step)
        .service(batch_update_approval_steps)
        .service(get_approval_step)
        .service(update_approval_step)
        .service(delete_approval_step);
}

/// A program's approval steps in display order. There are few of them, so the list is not paged.
#[instrument(skip(data))]
#[get("/{program}/approval-steps")]
pub async fn list_approval_steps(
    data: web::Data<AppState>,
    path: web::Path<ListApprovalStepsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListApprovalStepsController called");

    let usecase_input = ListApprovalStepsMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .approval_step
        .list_approval_steps
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListApprovalStepsMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[post("/{program}/approval-steps")]
pub async fn create_approval_step(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreateApprovalStepPathPayload>,
    query: web::Query<CreateApprovalStepQueryPayload>,
    body: web::Json<CreateApprovalStepBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateApprovalStepController called");

    let usecase_input =
        CreateApprovalStepMapper::input(path.into_inner(), query.into_inner(), body.into_inner(), actor_from(&req))
            .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .approval_step
        .create_approval_step
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateApprovalStepMapper::output_success(result)))
}

/// Sets the order of many approval steps in one request.
#[instrument(skip(data, req, body))]
#[post("/{program}/approval-steps:batchUpdate")]
pub async fn batch_update_approval_steps(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<BatchUpdateApprovalStepsPathPayload>,
    body: web::Json<BatchUpdateApprovalStepsBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("BatchUpdateApprovalStepsController called");

    let usecase_input = BatchUpdateApprovalStepsMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .approval_step
        .batch_update_approval_steps
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(BatchUpdateApprovalStepsMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/approval-steps/{approval_step}")]
pub async fn get_approval_step(
    data: web::Data<AppState>,
    path: web::Path<GetApprovalStepPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetApprovalStepController called");

    let usecase_input = GetApprovalStepMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .approval_step
        .get_approval_step
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetApprovalStepMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[patch("/{program}/approval-steps/{approval_step}")]
pub async fn update_approval_step(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateApprovalStepPathPayload>,
    body: web::Json<UpdateApprovalStepBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateApprovalStepController called");

    let usecase_input = UpdateApprovalStepMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .approval_step
        .update_approval_step
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateApprovalStepMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[delete("/{program}/approval-steps/{approval_step}")]
pub async fn delete_approval_step(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DeleteApprovalStepPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("DeleteApprovalStepController called");

    let usecase_input = DeleteApprovalStepMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    data.use_cases
        .approval_step
        .delete_approval_step
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::NoContent().finish())
}
