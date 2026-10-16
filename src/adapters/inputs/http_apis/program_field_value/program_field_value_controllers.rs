use crate::adapters::inputs::http_apis::program_field_value::program_field_value_mappers::{
    BatchUpdateFieldValuesMapper, ListFieldValuesMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::program_field_value::{
    BatchUpdateFieldValuesBodyPayload, BatchUpdateFieldValuesPathPayload, ListFieldValuesPathPayload,
    ListFieldValuesQueryPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_field_values).service(batch_update_field_values);
}

/// The saved values of a program's settings, with the version to send back on a save.
#[instrument(skip(data))]
#[get("/{program}/field-values")]
pub async fn list_field_values(
    data: web::Data<AppState>,
    path: web::Path<ListFieldValuesPathPayload>,
    query: web::Query<ListFieldValuesQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListFieldValuesController called");

    let usecase_input = ListFieldValuesMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program_field_value
        .list_field_values
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListFieldValuesMapper::output_success(result)))
}

/// Saves everything changed on a page ("N unsaved changes" -> Save) in ONE request and ONE transaction.
#[instrument(skip(data, req, body))]
#[post("/{program}/field-values:batchUpdate")]
pub async fn batch_update_field_values(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<BatchUpdateFieldValuesPathPayload>,
    body: web::Json<BatchUpdateFieldValuesBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("BatchUpdateFieldValuesController called");

    let usecase_input = BatchUpdateFieldValuesMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program_field_value
        .batch_update_field_values
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(BatchUpdateFieldValuesMapper::output_success(result)))
}
