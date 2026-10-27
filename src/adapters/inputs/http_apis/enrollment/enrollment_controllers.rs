use crate::adapters::inputs::http_apis::enrollment::enrollment_mappers::{
    BatchCreateEnrollmentsMapper, CreateEnrollmentMapper, GetEnrollmentMapper, ListEnrollmentsMapper,
    UpdateEnrollmentMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::enrollment::{
    BatchCreateEnrollmentsBodyPayload, BatchCreateEnrollmentsPathPayload, CreateEnrollmentBodyPayload,
    CreateEnrollmentPathPayload, CreateEnrollmentQueryPayload, GetEnrollmentPayload, ListEnrollmentsPathPayload,
    ListEnrollmentsQueryPayload, UpdateEnrollmentBodyPayload, UpdateEnrollmentPathPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, patch, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_enrollments)
        .service(create_enrollment)
        .service(batch_create_enrollments)
        .service(get_enrollment)
        .service(update_enrollment);
}

/// The Enrolled Stores page: newest first, with the "All websites" filter, status chips, a store-name
/// search and the "N STORES" count (`total_size`).
#[instrument(skip(data))]
#[get("/{program}/enrollments")]
pub async fn list_enrollments(
    data: web::Data<AppState>,
    path: web::Path<ListEnrollmentsPathPayload>,
    query: web::Query<ListEnrollmentsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListEnrollmentsController called");

    let usecase_input = ListEnrollmentsMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .enrollment
        .list_enrollments
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListEnrollmentsMapper::output_success(result)))
}

/// "Enroll store": an admin enrolls a store directly, without an application.
#[instrument(skip(data, req, body))]
#[post("/{program}/enrollments")]
pub async fn create_enrollment(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreateEnrollmentPathPayload>,
    query: web::Query<CreateEnrollmentQueryPayload>,
    body: web::Json<CreateEnrollmentBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateEnrollmentController called");

    let usecase_input =
        CreateEnrollmentMapper::input(path.into_inner(), query.into_inner(), body.into_inner(), actor_from(&req))
            .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .enrollment
        .create_enrollment
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateEnrollmentMapper::output_success(result)))
}

/// "Bulk enroll": many stores in one request; every item gets its own outcome.
#[instrument(skip(data, req, body))]
#[post("/{program}/enrollments:batchCreate")]
pub async fn batch_create_enrollments(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<BatchCreateEnrollmentsPathPayload>,
    body: web::Json<BatchCreateEnrollmentsBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("BatchCreateEnrollmentsController called");

    let usecase_input = BatchCreateEnrollmentsMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .enrollment
        .batch_create_enrollments
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(BatchCreateEnrollmentsMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/enrollments/{enrollment}")]
pub async fn get_enrollment(
    data: web::Data<AppState>,
    path: web::Path<GetEnrollmentPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetEnrollmentController called");

    let usecase_input = GetEnrollmentMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .enrollment
        .get_enrollment
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetEnrollmentMapper::output_success(result)))
}

/// The status dropdown (and the customer group).
#[instrument(skip(data, req, body))]
#[patch("/{program}/enrollments/{enrollment}")]
pub async fn update_enrollment(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateEnrollmentPathPayload>,
    body: web::Json<UpdateEnrollmentBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateEnrollmentController called");

    let usecase_input = UpdateEnrollmentMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .enrollment
        .update_enrollment
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateEnrollmentMapper::output_success(result)))
}
