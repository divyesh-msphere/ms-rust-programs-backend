use crate::adapters::inputs::http_apis::application::application_controllers;
use crate::adapters::inputs::http_apis::approval_step::approval_step_controllers;
use crate::adapters::inputs::http_apis::danger_zone::danger_zone_controllers;
use crate::adapters::inputs::http_apis::notifications::notifications_controllers;
use crate::adapters::inputs::http_apis::document::document_controllers;
use crate::adapters::inputs::http_apis::enrollment::enrollment_controllers;
use crate::adapters::inputs::http_apis::health::health_controllers;
use crate::adapters::inputs::http_apis::module_catalog_item::module_catalog_item_controllers;
use crate::adapters::inputs::http_apis::program::program_controllers;
use crate::adapters::inputs::http_apis::program_setting_value::program_setting_value_controllers;
use crate::adapters::inputs::http_apis::program_module::program_module_controllers;
use crate::adapters::inputs::http_apis::program_overview::program_overview_controllers;
use crate::adapters::inputs::http_apis::requirement::requirement_controllers;
use crate::adapters::inputs::http_apis::publication::publication_controllers;
use crate::adapters::inputs::http_apis::rule_set::rule_set_controllers;
use crate::adapters::inputs::http_apis::signoff::signoff_controllers;
use crate::adapters::inputs::http_apis::store_discount::store_discount_controllers;
use actix_web::web;

/// `enrollment_review` serves Requirements, Approval steps, Applications and Signoffs. Off (the
/// default, see `[features]` in config.toml) their paths do not exist and answer 404.
pub fn routes(enrollment_review: bool) -> impl Fn(&mut web::ServiceConfig) {
    move |cfg| configure(cfg, enrollment_review)
}

fn configure(cfg: &mut web::ServiceConfig, enrollment_review: bool) {
    cfg.service(web::scope("/health").configure(health_controllers::routes));
    cfg.service(web::scope("/module-catalog").configure(module_catalog_item_controllers::routes));
    // Custom methods (`:undelete`) sit outside the scope — see program_controllers.
    cfg.service(web::scope("/webhooks").configure(document_controllers::webhook_routes));
    cfg.configure(program_controllers::root_routes);
    cfg.service(
        web::scope("/programs")
            .configure(program_controllers::routes)
            .configure(program_module_controllers::routes)
            .configure(program_setting_value_controllers::routes)
            .configure(program_overview_controllers::routes)
            .configure(danger_zone_controllers::routes)
            .configure(notifications_controllers::routes)
            .configure(move |cfg| {
                if enrollment_review {
                    requirement_controllers::routes(cfg);
                    approval_step_controllers::routes(cfg);
                    application_controllers::routes(cfg);
                    signoff_controllers::routes(cfg);
                }
            })
            .configure(enrollment_controllers::routes)
            .configure(store_discount_controllers::routes)
            .configure(rule_set_controllers::routes)
            .configure(publication_controllers::routes)
            .configure(document_controllers::routes),
    );
}
