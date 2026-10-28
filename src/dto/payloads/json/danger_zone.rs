use serde::Deserialize;

/// The path of every Danger zone call.
#[derive(Debug, Deserialize, Clone)]
pub struct DangerZonePayload {
    /// The program's ID or its slug.
    pub program: String,
}
