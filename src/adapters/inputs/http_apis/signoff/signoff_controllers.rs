use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::http_apis::signoff::signoff_mappers::{CreateSignoffMapper, GetSignoffMapper, ListSignoffsMapper};
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::signoff::{
    CreateSignoffBodyPayload, CreateSignoffPathPayload, GetSignoffPayload, ListSignoffsPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_signoffs).service(create_signoff).service(get_signoff);
}

/// The application's decisions, in the order they were made. An application has at most one per
/// approval step, so the list is never paged.
#[instrument(skip(data))]
#[get("/{program}/applications/{application}/signoffs")]
pub async fn list_signoffs(
    data: web::Data<AppState>,
    path: web::Path<ListSignoffsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListSignoffsController called");

    let usecase_input = ListSignoffsMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .signoff
        .list_signoffs
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListSignoffsMapper::output_success(result)))
}

/// Approves or rejects the step the application is waiting at. It moves the application on to the
/// next step, approves it (last step) or rejects it.
#[instrument(skip(data, req, body))]
#[post("/{program}/applications/{application}/signoffs")]
pub async fn create_signoff(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreateSignoffPathPayload>,
    body: web::Json<CreateSignoffBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateSignoffController called");

    let usecase_input = CreateSignoffMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .signoff
        .create_signoff
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateSignoffMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/applications/{application}/signoffs/{signoff}")]
pub async fn get_signoff(
    data: web::Data<AppState>,
    path: web::Path<GetSignoffPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetSignoffController called");

    let usecase_input = GetSignoffMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .signoff
        .get_signoff
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetSignoffMapper::output_success(result)))
}
