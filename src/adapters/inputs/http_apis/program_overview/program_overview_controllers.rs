use crate::adapters::inputs::http_apis::program_overview::program_overview_mappers::{GetOverviewMapper, ListAuditLogsMapper};
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::program_overview::{GetOverviewPayload, ListAuditLogsPathPayload, ListAuditLogsQueryPayload};
use actix_web::{HttpResponse, get, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_overview).service(list_audit_logs);
}

/// The default page every program has: counters, the enrollment funnel, the published-rules sync
/// state and the latest activity, from one database round trip.
#[instrument(skip(data))]
#[get("/{program}/overview")]
pub async fn get_overview(
    data: web::Data<AppState>,
    path: web::Path<GetOverviewPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetOverviewController called");

    let usecase_input = GetOverviewMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program_overview
        .get_overview
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetOverviewMapper::output_success(result)))
}

/// A program's history, newest first: who changed what, with before and after.
#[instrument(skip(data))]
#[get("/{program}/audit-logs")]
pub async fn list_audit_logs(
    data: web::Data<AppState>,
    path: web::Path<ListAuditLogsPathPayload>,
    query: web::Query<ListAuditLogsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListAuditLogsController called");

    let usecase_input = ListAuditLogsMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .audit_log
        .list_audit_logs
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListAuditLogsMapper::output_success(result)))
}
