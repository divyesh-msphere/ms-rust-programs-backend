// domain/publication.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// A published, immutable snapshot of a program's configuration (the "bundle"): what pricing and the
/// other services load. Append-only; the newest one is what is live.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    pub id: Uuid,
    pub program_id: Uuid,
    /// The program's `config_version` this publication covers.
    pub config_version: i32,
    pub config_hash: String,
    /// What the bundle holds, in counts (modules, requirements, approval steps, store discounts) and
    /// the rule versions it carries. Always present.
    pub summary: Value,
    /// The whole bundle. Only on a single read.
    pub bundle: Option<Value>,
    pub created_by: Option<String>,
    pub create_time: DateTime<Utc>,
}

#[async_trait]
pub trait PublicationRepository: Send + Sync {
    /// Publishes the program's configuration. ONE transaction under the program lock, so the snapshot is
    /// consistent: the rule drafts become the published versions, the bundle is built and stored, and
    /// the audit entry and outbox event are written. `failed_precondition` when there is nothing new to
    /// publish, or when a rule's newest draft is invalid.
    async fn publish(&self, program_id: Uuid, actor: &str) -> Result<Publication, ApplicationError>;

    /// Newest first, keyset on the id. Rows carry no bundle.
    async fn list(
        &self,
        program_id: Uuid,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Publication>, Option<Uuid>), ApplicationError>;

    /// One publication with its bundle.
    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Publication, ApplicationError>;
}
