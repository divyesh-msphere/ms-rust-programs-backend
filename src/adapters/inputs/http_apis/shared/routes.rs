use crate::adapters::inputs::http_apis::health::health_controllers;
use crate::adapters::inputs::http_apis::module_definition::module_definition_controllers;
use crate::adapters::inputs::http_apis::program::program_controllers;
use crate::adapters::inputs::http_apis::program_field_value::program_field_value_controllers;
use crate::adapters::inputs::http_apis::program_module::program_module_controllers;
use crate::adapters::inputs::http_apis::webhook::webhook_controllers;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/health").configure(health_controllers::routes));
    cfg.service(web::scope("/module-definitions").configure(module_definition_controllers::routes));
    // Custom methods (`:undelete`) sit outside the scope — see program_controllers.
    cfg.service(web::scope("/webhooks").configure(webhook_controllers::routes));
    cfg.configure(program_controllers::root_routes);
    cfg.service(
        web::scope("/programs")
            .configure(program_controllers::routes)
            .configure(program_module_controllers::routes)
            .configure(program_field_value_controllers::routes),
    );
}
