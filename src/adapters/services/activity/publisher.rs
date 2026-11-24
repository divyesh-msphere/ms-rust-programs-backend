use super::subject;
use ms_telemetry_rs::{EventPublisher, Payload, TelemetryConfig};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Ships activity payloads to NATS; never blocks or fails a request.
pub struct ActivityPublisher {
    transport: Option<Arc<EventPublisher>>,
}

impl ActivityPublisher {
    /// Connects when `[nats] enabled = true` and the subject matches this service; otherwise
    /// the publisher only logs what it would have sent.
    pub async fn new(cfg: &TelemetryConfig) -> Self {
        if !cfg.nats.enabled {
            info!("activity: NATS disabled, events are only logged");
            return Self { transport: None };
        }
        let expected = subject();
        if cfg.nats.stream.subject != expected {
            error!(
                configured = %cfg.nats.stream.subject,
                expected = %expected,
                "activity: the configured NATS subject does not match, events are only logged"
            );
            return Self { transport: None };
        }
        match EventPublisher::new(&cfg.nats).await {
            Ok(publisher) => {
                info!(subject = %expected, "activity: NATS publisher connected");
                Self { transport: Some(Arc::new(publisher)) }
            }
            Err(err) => {
                warn!("activity: NATS connection failed ({err}), events are only logged");
                Self { transport: None }
            }
        }
    }

    pub fn publish(&self, mut payload: Payload) {
        // The activity store's TIMEUUID column silently drops anything but a v1 id.
        payload.id = new_activity_id();
        match &self.transport {
            Some(nats) => {
                let nats = Arc::clone(nats);
                actix_web::rt::spawn(async move { nats.publish(payload).await });
            }
            None => debug!(
                action = %payload.action,
                resource_type = %payload.resource_type,
                resource_id = %payload.resource_id,
                principal = %payload.principal_id,
                "activity event (not published)"
            ),
        }
    }
}

fn new_activity_id() -> Uuid {
    let random = Uuid::new_v4();
    let mut node = [0u8; 6];
    node.copy_from_slice(&random.as_bytes()[..6]);
    node[0] |= 0x01; // RFC 4122: a random node id sets the multicast bit
    Uuid::now_v1(&node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_v1() {
        assert_eq!(new_activity_id().get_version_num(), 1);
    }
}
