//! Activity-logger events: one event for every successful create, patch and delete.
//!
//! The HTTP middleware in [`middleware`] builds a `ms_telemetry_rs::Payload` from the request
//! (headers, resource type, resource id, service name) and hands it to [`ActivityPublisher`],
//! which ships it to NATS JetStream. When NATS is off or unreachable the payload is only logged.
pub mod middleware;
pub mod publisher;

/// `service_name` on every record. Chosen once: it is a partition key of the activity store.
pub const SERVICE_NAME: &str = "ms-programs";

/// The JetStream subject (`events.activity.>` needs at least one trailing token).
pub fn subject() -> String {
    format!("events.activity.{SERVICE_NAME}")
}
