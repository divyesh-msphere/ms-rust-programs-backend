use crate::dto::presenters::json::program_module::ProgramModulePresenter;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ProgramPresenter {
    pub path: String,
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub level: String,
    pub active: bool,
    pub config_version: i32,
    pub config_hash: String,
    pub create_time: String,
    pub update_time: String,
    pub state: String,
    /// List rows only.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub stats: Option<ProgramStatsPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnrollmentCountsPresenter {
    pub active: i32,
    pub total: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct DomainPresenter {
    pub website_id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DomainsPresenter {
    pub count: usize,
    pub items: Vec<DomainPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LastActivityPresenter {
    /// The `x-user-id` of the caller who made the newest change ("system" when none was sent).
    pub actor: String,
    pub action: String,
    pub summary: String,
    pub time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProgramStatsPresenter {
    pub enrollments: EnrollmentCountsPresenter,
    pub pending_approvals: i32,
    pub domains: DomainsPresenter,
    pub last_activity: Option<LastActivityPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProgramListSummaryPresenter {
    pub live_programs: i64,
    pub not_live_programs: i64,
    pub active_enrollments: i64,
    pub total_enrollments: i64,
    pub pending_approvals: i64,
    pub programs_with_pending_approvals: i64,
}

/// Create's (and, when modules changed, update's) response: the program plus the module manifest.
#[derive(Debug, Serialize, Clone)]
pub struct ProgramWithModulesPresenter {
    #[serde(flatten)]
    pub program: ProgramPresenter,
    /// Present on create, and on update when the request touched modules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<ProgramModulePresenter>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListProgramsPresenter {
    pub summary: ProgramListSummaryPresenter,
    pub results: Vec<ProgramPresenter>,
    pub next_page_token: Option<String>,
    pub total_size: i64,
}
