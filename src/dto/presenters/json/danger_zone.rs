use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DeleteBlockersPresenter {
    pub enrollments: i64,
    pub applications: i64,
    pub publications: i64,
    pub documents: i64,
}

/// The Danger zone page of a program.
#[derive(Debug, Serialize, Clone)]
pub struct DangerZonePresenter {
    pub path: String,
    pub archived: bool,
    pub archive_time: Option<String>,
    pub can_archive: bool,
    pub can_unarchive: bool,
    /// True only when the program has no operational history.
    pub can_delete: bool,
    pub delete_blockers: DeleteBlockersPresenter,
    /// Why it cannot be deleted; null when it can.
    pub delete_blocked_reason: Option<String>,
}
