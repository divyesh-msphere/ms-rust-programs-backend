use actix_web::{HttpRequest, HttpResponse};

pub async fn not_found_handler(req: HttpRequest) -> HttpResponse {
    let problem = serde_json::json!({
        "type": "NOT_FOUND",
        "status": 404,
        "title": "Not Found",
        "detail": format!("No route available: {} {}", req.method(), req.path()),
        "instance": uuid::Uuid::new_v4().to_string(),
    });
    HttpResponse::NotFound()
        .content_type("application/problem+json")
        .json(problem)
}
