use crate::adapters::inputs::http_apis::danger_zone::danger_zone_mappers::DangerZoneMapper;
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::danger_zone::DangerZonePayload;
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_danger_zone)
        .service(archive_program)
        .service(unarchive_program)
        .service(delete_program_permanently);
}

/// The premade Settings page every program has: is it archived, and what may each button do.
#[instrument(skip(data, req))]
#[get("/{program}/danger-zone")]
pub async fn get_danger_zone(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DangerZonePayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetDangerZoneController called");

    let usecase_input =
        DangerZoneMapper::input(path.into_inner(), actor_from(&req)).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .danger_zone
        .get_danger_zone
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(DangerZoneMapper::output_success(result)))
}

/// Takes the program out of the working catalog; reversible, nothing is lost.
#[instrument(skip(data, req))]
#[post("/{program}/danger-zone:archive")]
pub async fn archive_program(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DangerZonePayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ArchiveProgramController called");

    let usecase_input =
        DangerZoneMapper::input(path.into_inner(), actor_from(&req)).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .danger_zone
        .archive_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(DangerZoneMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[post("/{program}/danger-zone:unarchive")]
pub async fn unarchive_program(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DangerZonePayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UnarchiveProgramController called");

    let usecase_input =
        DangerZoneMapper::input(path.into_inner(), actor_from(&req)).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .danger_zone
        .unarchive_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(DangerZoneMapper::output_success(result)))
}

/// Removes the program for good. 409 while it has operational history.
#[instrument(skip(data, req))]
#[post("/{program}/danger-zone:deletePermanently")]
pub async fn delete_program_permanently(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DangerZonePayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("DeleteProgramPermanentlyController called");

    let usecase_input =
        DangerZoneMapper::input(path.into_inner(), actor_from(&req)).map_err(ErrorResponse::map_response_error)?;
    data.use_cases
        .danger_zone
        .delete_program_permanently
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::NoContent().finish())
}
