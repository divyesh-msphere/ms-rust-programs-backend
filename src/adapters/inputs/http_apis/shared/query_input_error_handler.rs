use actix_web::error::QueryPayloadError;
use actix_web::{HttpRequest, HttpResponse, error};
use ms_telemetry_rs::tracing::error;

pub fn query_error_handler(err: QueryPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let error_message = err.to_string();
    error!("Error in query string: {}", error_message);
    let problem = serde_json::json!({
        "type": "INVALID_ARGUMENT",
        "status": 400,
        "title": "Bad Request",
        "detail": format!("Invalid query parameter: {error_message}"),
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
