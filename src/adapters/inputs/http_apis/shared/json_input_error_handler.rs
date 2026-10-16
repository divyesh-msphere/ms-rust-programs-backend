use actix_web::error::JsonPayloadError;
use actix_web::{HttpRequest, HttpResponse, error};
use ms_telemetry_rs::tracing::error;

pub fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let error_message = err.to_string();
    error!("Error in JSON payload: {}", error_message);
    let problem = serde_json::json!({
        "type": "INVALID_ARGUMENT",
        "status": 400,
        "title": "Bad Request",
        // The parser's message names the field or position that is wrong (for example
        // "unknown field `setting`"), which a payload this rich needs to be usable.
        "detail": format!("Invalid JSON input: {error_message}"),
        "instance": uuid::Uuid::new_v4().to_string(),
    });
    error::InternalError::from_response(
        err,
        HttpResponse::BadRequest()
            .content_type("application/problem+json")
            .json(problem),
    )
    .into()
}
