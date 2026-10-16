use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::errors::application_error::ApplicationError;
use actix_web::{HttpRequest, HttpResponse, post, web};
use ms_telemetry_rs::tracing::{info, instrument};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(global_settings_event);
}

/// The envelope global-settings posts: `{event_id, event_type, event_payload}`.
#[derive(Debug, Deserialize)]
pub struct GlobalSettingsEvent {
    #[allow(dead_code)]
    pub event_id: Option<String>,
    pub event_type: String,
    #[serde(default)]
    pub event_payload: Value,
}

/// The library an event is about, when the payload says (global-settings sends the raw request
/// body, which usually does NOT carry it).
fn library_id_of(payload: &Value) -> Option<Uuid> {
    ["library_id", "id"]
        .iter()
        .find_map(|key| payload.get(key).and_then(Value::as_str))
        .and_then(|raw| Uuid::parse_str(raw).ok())
}

/// Called by global-settings when a library, one of its attributes, or one of its options changed
/// (`lib.*` events). What we cached of it is dropped and re-read, so a new or renamed option shows
/// up in the very next page load. Always answers 204 for an event it does not care about, so
/// global-settings never retries or logs a failure for it.
#[instrument(skip(data, req))]
#[post("/global-settings")]
pub async fn global_settings_event(
    req: HttpRequest,
    data: web::Data<AppState>,
    body: web::Json<GlobalSettingsEvent>,
) -> Result<HttpResponse, ErrorResponse> {
    let secret = &data.application_config.global_settings.webhook_secret;
    if !secret.is_empty() {
        let given = req.headers().get("x-webhook-secret").and_then(|v| v.to_str().ok()).unwrap_or_default();
        if given != secret {
            return Err(ErrorResponse::map_response_error(ApplicationError::unauthenticated(
                "invalid or missing X-Webhook-Secret",
            )));
        }
    }

    let event = body.into_inner();
    if event.event_type.starts_with("lib.") {
        data.library_catalog.invalidate(library_id_of(&event.event_payload));
        info!(event_type = %event.event_type, "global-settings library cache invalidated");
    }

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn finds_a_library_id_when_the_payload_has_one() {
        let id = Uuid::new_v4();
        assert_eq!(library_id_of(&json!({ "library_id": id.to_string() })), Some(id));
        assert_eq!(library_id_of(&json!({ "id": id.to_string(), "name": "x" })), Some(id));
        assert_eq!(library_id_of(&json!({ "code": "apple", "name": "Apple" })), None);
        assert_eq!(library_id_of(&json!({ "library_id": "nope" })), None);
        assert_eq!(library_id_of(&Value::Null), None);
    }
}
