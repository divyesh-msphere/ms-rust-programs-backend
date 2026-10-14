use crate::adapters::inputs::http_apis::health::health_controllers;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/health").configure(health_controllers::routes));
}
