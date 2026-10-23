use crate::adapters::inputs::http_apis::application::application_mappers::{
    CreateApplicationMapper, GetApplicationMapper, ListApplicationsMapper, SubmitApplicationMapper,
    WithdrawApplicationMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::application::{
    CreateApplicationBodyPayload, CreateApplicationPathPayload, CreateApplicationQueryPayload, GetApplicationPayload,
    ListApplicationsPathPayload, ListApplicationsQueryPayload, SubmitApplicationPayload, WithdrawApplicationPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_applications)
        .service(create_application)
        .service(get_application)
        .service(submit_application)
        .service(withdraw_application);
}

/// The Applications page: newest first, one status chip at a time (or several, comma-separated).
#[instrument(skip(data))]
#[get("/{program}/applications")]
pub async fn list_applications(
    data: web::Data<AppState>,
    path: web::Path<ListApplicationsPathPayload>,
    query: web::Query<ListApplicationsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListApplicationsController called");

    let usecase_input = ListApplicationsMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .application
        .list_applications
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListApplicationsMapper::output_success(result)))
}

/// A store starts an application: it begins as a draft.
#[instrument(skip(data, req, body))]
#[post("/{program}/applications")]
pub async fn create_application(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreateApplicationPathPayload>,
    query: web::Query<CreateApplicationQueryPayload>,
    body: web::Json<CreateApplicationBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateApplicationController called");

    let usecase_input =
        CreateApplicationMapper::input(path.into_inner(), query.into_inner(), body.into_inner(), actor_from(&req))
            .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .application
        .create_application
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateApplicationMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/applications/{application}")]
pub async fn get_application(
    data: web::Data<AppState>,
    path: web::Path<GetApplicationPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetApplicationController called");

    let usecase_input = GetApplicationMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .application
        .get_application
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetApplicationMapper::output_success(result)))
}

/// draft -> submitted: freezes the requirements and enters the first approval step.
#[instrument(skip(data, req))]
#[post("/{program}/applications/{application}:submit")]
pub async fn submit_application(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<SubmitApplicationPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("SubmitApplicationController called");

    let usecase_input = SubmitApplicationMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .application
        .submit_application
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(SubmitApplicationMapper::output_success(result)))
}

/// A draft, submitted or in-review application is withdrawn; a decided one cannot be.
#[instrument(skip(data, req))]
#[post("/{program}/applications/{application}:withdraw")]
pub async fn withdraw_application(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<WithdrawApplicationPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("WithdrawApplicationController called");

    let usecase_input = WithdrawApplicationMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .application
        .withdraw_application
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(WithdrawApplicationMapper::output_success(result)))
}
