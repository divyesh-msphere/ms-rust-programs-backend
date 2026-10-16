use actix_web::HttpRequest;

/// Who is making the change, for `audit_log.actor`. The gateway forwards the
/// authenticated user as `X-User-Id`; a direct call without it is recorded as
/// `system` rather than failing.
pub fn actor_from(req: &HttpRequest) -> String {
    req.headers()
        .get("x-user-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("system")
        .to_string()
}
