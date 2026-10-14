use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::utils::datetime::format_datetime;
use actix_web::{HttpResponse, get, web};
use chrono::Utc;
use ms_telemetry_rs::tracing::instrument;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct HealthCheckPresenter {
    status: String,
    service: String,
    check_time: String,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_health);
}

#[instrument(skip(data))]
#[get("")]
pub async fn get_health(data: web::Data<AppState>) -> HttpResponse {
    // info!("GetHealthController called");

    HttpResponse::Ok().json(HealthCheckPresenter {
        status: "ok".to_string(),
        service: data.application_config.app_name.clone(),
        check_time: format_datetime(Utc::now()),
    })
}
