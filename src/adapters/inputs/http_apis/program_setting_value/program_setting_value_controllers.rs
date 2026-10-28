use crate::adapters::inputs::http_apis::program_setting_value::program_setting_value_mappers::{
    BatchUpdateSettingValuesMapper, ListSettingValuesMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::program_setting_value::{
    BatchUpdateSettingValuesBodyPayload, BatchUpdateSettingValuesPathPayload, ListSettingValuesPathPayload,
    ListSettingValuesQueryPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_setting_values).service(batch_update_setting_values);
}

/// The saved values of a program's settings, with the version to send back on a save.
#[instrument(skip(data))]
#[get("/{program}/setting-values")]
pub async fn list_setting_values(
    data: web::Data<AppState>,
    path: web::Path<ListSettingValuesPathPayload>,
    query: web::Query<ListSettingValuesQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListSettingValuesController called");

    let usecase_input = ListSettingValuesMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program_setting_value
        .list_setting_values
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListSettingValuesMapper::output_success(result)))
}

/// Saves everything changed on a page ("N unsaved changes" -> Save) in ONE request and ONE transaction.
#[instrument(skip(data, req, body))]
#[post("/{program}/setting-values:batchUpdate")]
pub async fn batch_update_setting_values(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<BatchUpdateSettingValuesPathPayload>,
    body: web::Json<BatchUpdateSettingValuesBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("BatchUpdateSettingValuesController called");

    let usecase_input = BatchUpdateSettingValuesMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program_setting_value
        .batch_update_setting_values
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(BatchUpdateSettingValuesMapper::output_success(result)))
}
