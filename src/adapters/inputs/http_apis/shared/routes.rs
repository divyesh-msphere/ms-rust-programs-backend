use crate::adapters::inputs::http_apis::health::health_controllers;
use crate::adapters::inputs::http_apis::module_definition::module_definition_controllers;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/health").configure(health_controllers::routes));
    cfg.service(web::scope("/module-definitions").configure(module_definition_controllers::routes));
}
