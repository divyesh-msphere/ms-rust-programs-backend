// domain/danger_zone.rs
//
// The Danger zone page of a program (Settings): archive (reversible, the data stays) and delete
// permanently (only when the program has no operational history).
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// The operational history of a program. Anything above 0 means the program was used, so it can be
/// archived but not deleted for good.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeleteBlockers {
    pub enrollments: i64,
    pub applications: i64,
    pub publications: i64,
    /// Documents point at files in object storage; deleting the rows would leave the files behind.
    pub documents: i64,
}

impl DeleteBlockers {
    pub fn total(&self) -> i64 {
        self.enrollments + self.applications + self.publications + self.documents
    }

    /// "it has 3 enrollments and 1 publication", or `None` when there is no history.
    pub fn reason(&self) -> Option<String> {
        let parts: Vec<String> = [
            (self.enrollments, "enrollment"),
            (self.applications, "application"),
            (self.publications, "publication"),
            (self.documents, "document"),
        ]
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .map(|(count, noun)| format!("{count} {noun}{}", if count == 1 { "" } else { "s" }))
        .collect();
        if parts.is_empty() {
            None
        } else {
            Some(format!("it has operational history ({}); archive it instead", parts.join(", ")))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DangerZone {
    pub program_id: Uuid,
    pub archive_time: Option<DateTime<Utc>>,
    pub blockers: DeleteBlockers,
}

impl DangerZone {
    /// The AEP path of the page.
    pub fn path(&self) -> String {
        format!("programs/{}/danger-zone", self.program_id)
    }

    pub fn archived(&self) -> bool {
        self.archive_time.is_some()
    }

    pub fn can_archive(&self) -> bool {
        !self.archived()
    }

    pub fn can_unarchive(&self) -> bool {
        self.archived()
    }

    pub fn can_delete(&self) -> bool {
        self.blockers.total() == 0
    }
}

#[async_trait]
pub trait DangerZoneRepository: Send + Sync {
    /// The page of a live program.
    async fn get(&self, program_id: Uuid) -> Result<DangerZone, ApplicationError>;
    /// Takes the program out of the working catalog. 400 when it is already archived.
    async fn archive(&self, program_id: Uuid, actor: &str) -> Result<DangerZone, ApplicationError>;
    /// Puts it back. 400 when it is not archived.
    async fn unarchive(&self, program_id: Uuid, actor: &str) -> Result<DangerZone, ApplicationError>;
    /// Removes the program and everything that belongs to it. 409 while it has operational history.
    async fn delete_permanently(&self, program_id: Uuid, actor: &str) -> Result<(), ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_history_means_it_can_be_deleted() {
        let blockers = DeleteBlockers::default();
        assert_eq!(blockers.total(), 0);
        assert_eq!(blockers.reason(), None);
    }

    #[test]
    fn the_reason_names_what_blocks_it() {
        let blockers = DeleteBlockers { enrollments: 3, publications: 1, ..Default::default() };
        assert_eq!(
            blockers.reason().as_deref(),
            Some("it has operational history (3 enrollments, 1 publication); archive it instead")
        );
    }

    #[test]
    fn archived_flips_the_buttons() {
        let mut page = DangerZone { program_id: Uuid::nil(), archive_time: None, blockers: DeleteBlockers::default() };
        assert!(page.can_archive() && !page.can_unarchive() && page.can_delete());
        page.archive_time = Some(Utc::now());
        assert!(!page.can_archive() && page.can_unarchive());
    }
}
