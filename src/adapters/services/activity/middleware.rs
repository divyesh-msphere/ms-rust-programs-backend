use super::SERVICE_NAME;
use super::publisher::ActivityPublisher;
use actix_web::body::{BoxBody, MessageBody, to_bytes};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::Method;
use actix_web::middleware::Next;
use actix_web::{Error, HttpResponse, web};
use ms_telemetry_rs::{HeaderExtractor, Payload};

struct Headers<'a>(&'a actix_web::http::header::HeaderMap);

impl HeaderExtractor for Headers<'_> {
    fn get_header(&self, name: &str) -> Option<String> {
        self.0.get(name)?.to_str().ok().map(str::to_string)
    }
}

/// What a route is about: `/programs/{program}/documents/{document}:publish` is the resource
/// `documents`, identified by the `document` parameter, with the custom verb `publish`.
#[derive(Debug, PartialEq)]
struct RouteInfo {
    resource: String,
    id_param: Option<String>,
    verb: Option<String>,
}

fn parse_pattern(pattern: &str) -> Option<RouteInfo> {
    let mut info = RouteInfo { resource: String::new(), id_param: None, verb: None };
    for segment in pattern.split('/').filter(|s| !s.is_empty()) {
        let (name, verb) = match segment.split_once(':') {
            Some((name, verb)) if !name.is_empty() => (name, Some(verb)),
            _ => (segment, None),
        };
        if let Some(param) = name.strip_prefix('{').and_then(|n| n.strip_suffix('}')) {
            if !info.resource.is_empty() && info.id_param.is_none() {
                info.id_param = Some(param.to_string());
            }
        } else {    
            info.resource = name.to_string();
            info.id_param = None;
        }
        info.verb = verb.map(str::to_string);
    }
    (!info.resource.is_empty()).then_some(info)
}

fn singular(name: &str) -> String {
    let name = name.replace('-', "_");
    if let Some(stem) = name.strip_suffix("ies") {
        format!("{stem}y")
    } else if name.ends_with("ss") || name.ends_with("us") || name.ends_with("is") {
        name
    } else {
        name.strip_suffix('s').map(str::to_string).unwrap_or(name)
    }
}

fn event_key(resource: &str, method: &Method, verb: Option<&str>) -> String {
    let suffix = match verb {
        Some(verb) => verb.replace('-', "_"),
        None if *method == Method::POST => "created".to_string(),
        None if *method == Method::DELETE => "deleted".to_string(),
        None => "updated".to_string(),
    };
    format!("{}_{suffix}", singular(resource))
}

fn title(key: &str) -> String {
    key.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The id of what a create returned: the body's `id`, else the last segment of its `path`.
fn id_from_body(bytes: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let text = |key: &str| value.get(key).and_then(|v| v.as_str()).map(str::to_string);
    text("id").or_else(|| text("path").and_then(|p| p.rsplit('/').next().map(str::to_string)))
}

/// Publishes one activity event for every successful POST / PUT / PATCH / DELETE.
pub async fn activity_log(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let method = req.method().clone();
    if !matches!(method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE) {
        return next.call(req).await.map(|res| res.map_into_boxed_body());
    }
    let headers = req.headers().clone();
    let publisher = req.app_data::<web::Data<ActivityPublisher>>().cloned();
    let res = next.call(req).await?.map_into_boxed_body();
    let Some(publisher) = publisher else { return Ok(res) };
    if !res.status().is_success() {
        return Ok(res);
    }
    let Some(route) = res.request().match_pattern().and_then(|p| parse_pattern(&p)) else {
        return Ok(res);
    };

    let mut id = route.id_param.as_deref().and_then(|p| res.request().match_info().get(p)).map(str::to_string);
    let mut res = res;
    // A create has no id in the URL: read it from the response body.
    let is_json = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("json"));
    if id.is_none() && method == Method::POST && is_json {
        let (request, response) = res.into_parts();
        let (head, body) = response.into_parts();
        let bytes = to_bytes(body).await.map_err(|_| actix_web::error::ErrorInternalServerError("response body"))?;
        id = id_from_body(&bytes);
        res = ServiceResponse::new(request, HttpResponse::from(head.set_body(BoxBody::new(bytes))));
    }
    // A singleton (`/programs/{program}/overview`) is identified by its parent.
    if id.is_none() {
        id = res.request().match_info().iter().last().map(|(_, v)| v.to_string());
    }

    let key = event_key(&route.resource, &method, route.verb.as_deref());
    let mut payload = Payload::from_headers(
        &Headers(&headers),
        method.as_str(),
        SERVICE_NAME,
        &title(&key),
        &singular(&route.resource),
    );
    payload.action = key;
    payload.resource_id = id.unwrap_or_default();
    publisher.publish(payload);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_collection_item_and_custom_method_routes() {
        let r = parse_pattern("/programs/{program}/documents/{document}:publish").unwrap();
        assert_eq!((r.resource.as_str(), r.id_param.as_deref(), r.verb.as_deref()), ("documents", Some("document"), Some("publish")));
        let r = parse_pattern("/programs/{program}/documents").unwrap();
        assert_eq!((r.resource.as_str(), r.id_param), ("documents", None));
        let r = parse_pattern("/webhooks/document-scan:receive").unwrap();
        assert_eq!((r.resource.as_str(), r.verb.as_deref()), ("document-scan", Some("receive")));
    }

    #[test]
    fn builds_the_event_key() {
        assert_eq!(event_key("documents", &Method::POST, None), "document_created");
        assert_eq!(event_key("rule-sets", &Method::PATCH, None), "rule_set_updated");
        assert_eq!(event_key("categories", &Method::DELETE, None), "category_deleted");
        assert_eq!(event_key("documents", &Method::POST, Some("publish")), "document_publish");
        assert_eq!(title("document_created"), "Document Created");
    }

    #[test]
    fn reads_the_created_id() {
        assert_eq!(id_from_body(br#"{"id":"a1","path":"programs/p/x/b2"}"#).as_deref(), Some("a1"));
        assert_eq!(id_from_body(br#"{"path":"programs/p/x/b2"}"#).as_deref(), Some("b2"));
    }
}
