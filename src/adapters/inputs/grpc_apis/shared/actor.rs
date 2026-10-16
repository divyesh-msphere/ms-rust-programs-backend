use connectrpc::RequestContext;

/// Who is making the change, for `audit_log.actor`: the `x-user-id` header the
/// gateway forwards, or `system` for a direct call without one (same rule as
/// the HTTP side).
pub fn actor_from(ctx: &RequestContext) -> String {
    ctx.headers()
        .get("x-user-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("system")
        .to_string()
}
