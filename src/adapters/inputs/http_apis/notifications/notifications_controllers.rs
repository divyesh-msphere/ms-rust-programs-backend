use crate::adapters::inputs::http_apis::notifications::notifications_mappers::{GetNotificationsMapper, ReplaceNotificationsMapper};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::notifications::{NotificationsPathPayload, ReplaceNotificationsBodyPayload};
use actix_web::{HttpRequest, HttpResponse, get, put, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_notifications).service(replace_notifications);
}

/// The premade Notifications page: the trigger points and which channels each one uses.
#[instrument(skip(data))]
#[get("/{program}/notifications")]
pub async fn get_notifications(
    data: web::Data<AppState>,
    path: web::Path<NotificationsPathPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetNotificationsController called");

    let usecase_input = GetNotificationsMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .notifications
        .get_notifications
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetNotificationsMapper::output_success(result)))
}

/// Saves the whole grid (ticked boxes, added and removed triggers) in ONE request and ONE transaction.
#[instrument(skip(data, req, body))]
#[put("/{program}/notifications")]
pub async fn replace_notifications(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<NotificationsPathPayload>,
    body: web::Json<ReplaceNotificationsBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ReplaceNotificationsController called");

    let usecase_input = ReplaceNotificationsMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .notifications
        .replace_notifications
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ReplaceNotificationsMapper::output_success(result)))
}
