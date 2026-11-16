// domain/program.rs
use async_trait::async_trait;
use crate::domain::program_query::{ListFilter, SearchTerm};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::aep::{ResourceIdentifier, ResourceState};
use crate::domain::program_module::ProgramModule;
use crate::domain::program_setting::ModuleChange;

/// The customer-hierarchy grain a program is anchored at.
pub const PROGRAM_LEVELS: [&str; 3] = ["user", "organization", "location"];
pub const DEFAULT_PROGRAM_LEVEL: &str = "location";

/// The badge of a program: its colours and icon. Each part is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branding {
    /// `#RRGGBB`, the badge background.
    pub brand_color: Option<String>,
    /// `#RRGGBB`, the text on the badge.
    pub text_color: Option<String>,
    /// An icon name, e.g. `credit_card`.
    pub logo_icon: Option<String>,
}

/// The renewal defaults of a program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Renewal {
    pub term_months: i32,
    pub grace_period_days: i32,
    /// Days before the term ends.
    pub reminder_days: i32,
    pub auto_renew: bool,
}

impl Default for Renewal {
    fn default() -> Self {
        Self { term_months: 12, grace_period_days: 30, reminder_days: 30, auto_renew: false }
    }
}

/// A partial badge change: a sent part replaces that part (an empty text clears it), an absent
/// part is kept.
#[derive(Debug, Clone, Default)]
pub struct BrandingPatch {
    pub brand_color: Option<String>,
    pub text_color: Option<String>,
    pub logo_icon: Option<String>,
}

impl BrandingPatch {
    pub fn apply(self, current: &Branding) -> Branding {
        Branding {
            brand_color: self.brand_color.or_else(|| current.brand_color.clone()),
            text_color: self.text_color.or_else(|| current.text_color.clone()),
            logo_icon: self.logo_icon.or_else(|| current.logo_icon.clone()),
        }
    }
}

/// A partial renewal change: a sent part replaces that part, an absent part is kept.
#[derive(Debug, Clone, Default)]
pub struct RenewalPatch {
    pub term_months: Option<i32>,
    pub grace_period_days: Option<i32>,
    pub reminder_days: Option<i32>,
    pub auto_renew: Option<bool>,
}

impl RenewalPatch {
    pub fn apply(self, current: &Renewal) -> Renewal {
        Renewal {
            term_months: self.term_months.unwrap_or(current.term_months),
            grace_period_days: self.grace_period_days.unwrap_or(current.grace_period_days),
            reminder_days: self.reminder_days.unwrap_or(current.reminder_days),
            auto_renew: self.auto_renew.unwrap_or(current.auto_renew),
        }
    }
}

pub const MAX_HIGHLIGHTS: usize = 5;
const MAX_HIGHLIGHT_LEN: usize = 100;
const MAX_ABOUT_LEN: usize = 1000;
const MAX_ICON_LEN: usize = 64;
const MAX_TERM_MONTHS: i32 = 120;
const MAX_DAYS: i32 = 3650;

/// A program (e.g. KBB, CPO, Credit Line). `Serialize`/`Deserialize` are for a
/// future cache, not an API response shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: Uuid,                   // OUTPUT_ONLY
    pub name: String,
    pub slug: String,               // IMMUTABLE
    pub description: Option<String>,
    /// The merchant-facing text (`description` is the internal one).
    pub about: Option<String>,
    /// Up to 5 short selling points, in order.
    pub highlights: Vec<String>,
    pub branding: Branding,
    pub renewal: Renewal,
    pub level: String,
    pub active: bool,
    pub config_version: i32,        // OUTPUT_ONLY — bumped on every configuration change
    pub config_hash: String,        // OUTPUT_ONLY
    pub state: ResourceState,       // OUTPUT_ONLY (AEP-216)
    pub create_time: DateTime<Utc>, // OUTPUT_ONLY
    pub update_time: DateTime<Utc>, // OUTPUT_ONLY
}

/// What the programs list narrows to. The cards above the table ignore it.
#[derive(Debug, Clone, Default)]
pub struct ProgramFilter {
    pub show_deleted: bool,
    pub show_archived: bool,
    /// ANDed text conditions (`search=`).
    pub search: Vec<SearchTerm>,
    /// Keyed value lists (`filter=`).
    pub list: ListFilter,
}

/// A website (domain) a program has enrollments on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramDomain {
    pub website_id: String,
    pub name: Option<String>,
}

/// The newest audit entry of a program: who changed it last, and what.
#[derive(Debug, Clone)]
pub struct LastActivity {
    pub actor: String,
    pub action: String,
    pub target_type: String,
    pub time: DateTime<Utc>,
}

impl LastActivity {
    /// The line the table shows, e.g. "Program updated" or "Document created".
    pub fn summary(&self) -> String {
        let what = match self.target_type.as_str() {
            "program" => "Program".to_string(),
            "document_folder" => "Folder".to_string(),
            other => {
                let mut chars = other.replace('_', " ").chars().collect::<Vec<_>>();
                if let Some(first) = chars.first_mut() {
                    *first = first.to_ascii_uppercase();
                }
                chars.into_iter().collect()
            }
        };
        format!("{what} {}", self.action.replace('_', " "))
    }
}

/// The per-row numbers of the programs table.
#[derive(Debug, Clone, Default)]
pub struct ProgramListStats {
    pub active_enrollments: i32,
    pub total_enrollments: i32,
    pub pending_approvals: i32,
    pub domains: Vec<ProgramDomain>,
    pub last_activity: Option<LastActivity>,
}

/// One row of the list: the program and its numbers.
#[derive(Debug, Clone)]
pub struct ProgramListItem {
    pub program: Program,
    pub stats: ProgramListStats,
}

/// The cards above the table, over every program the filter matches (not just the page).
#[derive(Debug, Clone, Copy, Default)]
pub struct ProgramListSummary {
    /// Every program in the view: `live_programs + not_live_programs`.
    pub total_programs: i64,
    pub live_programs: i64,
    pub not_live_programs: i64,
    pub active_enrollments: i64,
    pub total_enrollments: i64,
    pub pending_approvals: i64,
    pub programs_with_pending_approvals: i64,
}

#[derive(Debug, Clone)]
pub struct NewProgram {
    /// Optional client-specified ID (AEP-133); server generates a UUID v7 if omitted.
    pub id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub about: Option<String>,
    pub highlights: Vec<String>,
    pub branding: Branding,
    pub renewal: Renewal,
    pub level: String,
    /// The modules the admin picked, each optionally with its mode and settings. Ancestors, the
    /// descendants of a picked group, and the default modules (Overview) are added by the
    /// repository.
    pub modules: Vec<ModuleChange>,
    pub actor: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateProgram {
    pub name: Option<String>,
    /// `Some(None)` clears the description.
    pub description: Option<Option<String>>,
    /// `Some(None)` clears the merchant-facing text.
    pub about: Option<Option<String>>,
    /// `Some` replaces the whole list.
    pub highlights: Option<Vec<String>>,
    /// `Some` replaces the badge.
    pub branding: Option<Branding>,
    /// `Some` replaces the renewal defaults.
    pub renewal: Option<Renewal>,
    pub level: Option<String>,
    pub active: Option<bool>,
    /// Module changes: a new key adds the module, `mode` sets what a leaf is, `settings`
    /// replaces its settings, `enabled: false` switches it off. `None` touches no module.
    pub modules: Option<Vec<ModuleChange>>,
    pub actor: String,
}

/// `slug` rule: lowercase letters and digits separated by single hyphens.
pub fn validate_slug(slug: &str) -> Result<(), ApplicationError> {
    let ok = !slug.is_empty()
        && slug.len() <= 64
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !ok {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "slug {slug:?} must be 1-64 lowercase letters, digits and single hyphens"
        )));
    }

    // `GET /programs/{program}` takes an id OR a slug: a slug that parses as a UUID
    // would be read as an id and never found by slug.
    if Uuid::parse_str(slug).is_ok() {
        return Err(ApplicationError::unprocessable_entity(
            "slug must not look like a UUID (it would be read as an id)",
        ));
    }

    Ok(())
}

fn unprocessable(message: String) -> ApplicationError {
    ApplicationError::unprocessable_entity(&message)
}

/// `#RRGGBB`.
fn valid_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Trims the merchant-facing text; empty reads as absent.
pub fn clean_about(about: Option<String>) -> Result<Option<String>, ApplicationError> {
    let about = about.map(|a| a.trim().to_string()).filter(|a| !a.is_empty());
    if let Some(text) = &about
        && text.chars().count() > MAX_ABOUT_LEN
    {
        return Err(unprocessable(format!("about must be at most {MAX_ABOUT_LEN} characters")));
    }
    Ok(about)
}

/// Trims each highlight and drops empty ones; at most 5 remain.
pub fn clean_highlights(highlights: Vec<String>) -> Result<Vec<String>, ApplicationError> {
    let cleaned: Vec<String> = highlights.into_iter().map(|h| h.trim().to_string()).filter(|h| !h.is_empty()).collect();
    if cleaned.len() > MAX_HIGHLIGHTS {
        return Err(unprocessable(format!("highlights takes at most {MAX_HIGHLIGHTS} items")));
    }
    if cleaned.iter().any(|h| h.chars().count() > MAX_HIGHLIGHT_LEN) {
        return Err(unprocessable(format!("a highlight must be at most {MAX_HIGHLIGHT_LEN} characters")));
    }
    Ok(cleaned)
}

/// Colours must be `#RRGGBB` (stored upper-case); an empty value reads as absent.
pub fn clean_branding(branding: Branding) -> Result<Branding, ApplicationError> {
    let color = |name: &str, value: Option<String>| -> Result<Option<String>, ApplicationError> {
        match value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
            None => Ok(None),
            Some(v) if valid_color(&v) => Ok(Some(v.to_uppercase())),
            Some(v) => Err(unprocessable(format!("branding.{name} {v:?} must look like #1B62F6"))),
        }
    };
    let logo_icon = branding.logo_icon.map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    if let Some(icon) = &logo_icon
        && (icon.chars().count() > MAX_ICON_LEN
            || !icon.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
    {
        return Err(unprocessable(format!(
            "branding.logo_icon must be an icon name of letters, digits, '_' or '-' (up to {MAX_ICON_LEN})"
        )));
    }
    Ok(Branding {
        brand_color: color("brand_color", branding.brand_color)?,
        text_color: color("text_color", branding.text_color)?,
        logo_icon,
    })
}

pub fn validate_renewal(renewal: &Renewal) -> Result<(), ApplicationError> {
    if !(1..=MAX_TERM_MONTHS).contains(&renewal.term_months) {
        return Err(unprocessable(format!("renewal.term_months must be 1-{MAX_TERM_MONTHS}")));
    }
    if !(0..=MAX_DAYS).contains(&renewal.grace_period_days) {
        return Err(unprocessable(format!("renewal.grace_period_days must be 0-{MAX_DAYS}")));
    }
    if !(0..=MAX_DAYS).contains(&renewal.reminder_days) {
        return Err(unprocessable(format!("renewal.reminder_days must be 0-{MAX_DAYS}")));
    }
    if renewal.reminder_days > renewal.term_months * 30 {
        return Err(unprocessable("renewal.reminder_days cannot be longer than the renewal term".to_string()));
    }
    Ok(())
}

pub fn validate_level(level: &str) -> Result<(), ApplicationError> {
    if PROGRAM_LEVELS.contains(&level) {
        Ok(())
    } else {
        Err(ApplicationError::unprocessable_entity(&format!(
            "level {level:?} must be one of: {}",
            PROGRAM_LEVELS.join(", ")
        )))
    }
}

#[async_trait]
pub trait ProgramRepository: Send + Sync {
    /// Accepts the id or the slug. Deleted programs are only found with
    /// `include_deleted` (Undelete needs them).
    async fn get(&self, identifier: &ResourceIdentifier, include_deleted: bool) -> Result<Program, ApplicationError>;

    /// `show_deleted` toggles between active-only (false) and deleted-only
    /// (true); `show_archived` between the working catalog (false: archived programs
    /// left out) and archived-only (true). The returned `i64` is `total_size` for the
    /// same filter.
    async fn list(
        &self,
        page_size: i64,
        page_token: Option<Uuid>,
        filter: &ProgramFilter,
    ) -> Result<(Vec<Program>, Option<Uuid>, i64), ApplicationError>;

    /// The numbers of the given programs, for the list rows.
    async fn list_stats(&self, ids: &[Uuid]) -> Result<std::collections::HashMap<Uuid, ProgramListStats>, ApplicationError>;

    /// The cards above the table, for the same filter as `list`.
    async fn list_summary(&self, show_deleted: bool, show_archived: bool) -> Result<ProgramListSummary, ApplicationError>;

    /// ONE transaction: the program, its modules, the Overview counters row,
    /// the audit entry and the outbox event.
    async fn create(&self, input: NewProgram) -> Result<(Program, Vec<ProgramModule>), ApplicationError>;

    /// One transaction with the audit entry and outbox event. When `input.modules` is given,
    /// the module manifest after the change comes back too.
    async fn update(
        &self,
        id: Uuid,
        input: UpdateProgram,
    ) -> Result<(Program, Option<Vec<ProgramModule>>), ApplicationError>;

    /// Soft delete (state -> DELETED).
    async fn delete(&self, id: Uuid, actor: &str) -> Result<(), ApplicationError>;

    /// Reverses a soft delete.
    async fn undelete(&self, id: Uuid, actor: &str) -> Result<Program, ApplicationError>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn branding_takes_hex_colours_and_an_icon_name() {
        let ok = clean_branding(Branding {
            brand_color: Some(" #1b62f6 ".into()),
            text_color: Some("#FFFFFF".into()),
            logo_icon: Some("credit_card".into()),
        })
        .unwrap();
        assert_eq!(ok.brand_color.as_deref(), Some("#1B62F6"));
        for bad in ["1B62F6", "#12345", "#GGGGGG", "blue"] {
            let b = Branding { brand_color: Some(bad.into()), ..Default::default() };
            assert!(clean_branding(b).is_err(), "{bad}");
        }
        assert!(clean_branding(Branding { logo_icon: Some("bad icon!".into()), ..Default::default() }).is_err());
        assert_eq!(clean_branding(Branding { brand_color: Some("  ".into()), ..Default::default() }).unwrap(), Branding::default());
    }

    #[test]
    fn highlights_are_trimmed_and_capped_at_five() {
        let h = clean_highlights(vec![" a ".into(), "".into(), "b".into()]).unwrap();
        assert_eq!(h, vec!["a", "b"]);
        assert!(clean_highlights((0..6).map(|i| i.to_string()).collect()).is_err());
        assert!(clean_highlights(vec!["x".repeat(101)]).is_err());
    }

    #[test]
    fn renewal_defaults_and_limits() {
        assert_eq!(Renewal::default(), Renewal { term_months: 12, grace_period_days: 30, reminder_days: 30, auto_renew: false });
        assert!(validate_renewal(&Renewal::default()).is_ok());
        assert!(validate_renewal(&Renewal { term_months: 0, ..Default::default() }).is_err());
        assert!(validate_renewal(&Renewal { grace_period_days: -1, ..Default::default() }).is_err());
        assert!(validate_renewal(&Renewal { term_months: 1, reminder_days: 31, ..Default::default() }).is_err());
    }

    #[test]
    fn a_patch_keeps_what_it_does_not_send() {
        let current = Renewal { term_months: 6, grace_period_days: 10, reminder_days: 5, auto_renew: true };
        let next = RenewalPatch { term_months: Some(24), ..Default::default() }.apply(&current);
        assert_eq!(next, Renewal { term_months: 24, grace_period_days: 10, reminder_days: 5, auto_renew: true });
        let b = Branding { brand_color: Some("#111111".into()), text_color: Some("#222222".into()), logo_icon: None };
        let merged = BrandingPatch { text_color: Some("#333333".into()), ..Default::default() }.apply(&b);
        assert_eq!(merged.brand_color.as_deref(), Some("#111111"));
        assert_eq!(merged.text_color.as_deref(), Some("#333333"));
    }

    use super::*;

    #[test]
    fn accepts_valid_slugs() {
        for slug in ["kbb", "cl1-isp", "a", "pre-owned-devices", "x9", &"a".repeat(64)] {
            assert!(validate_slug(slug).is_ok(), "{slug} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_slugs() {
        for slug in ["", "01a14355-dfba-7716-8068-d62f02de7088", "KBB", "Bad Slug", "-kbb", "kbb-", "kb--b", "kb_b", "kbb!", "ünï", &"a".repeat(65)] {
            assert!(validate_slug(slug).is_err(), "{slug:?} should be invalid");
        }
    }

    #[test]
    fn accepts_only_known_levels() {
        for level in PROGRAM_LEVELS {
            assert!(validate_level(level).is_ok());
        }
        assert!(validate_level("galaxy").is_err());
        assert!(validate_level("").is_err());
        assert!(validate_level("Location").is_err());
    }

    #[test]
    fn default_level_is_valid() {
        assert!(validate_level(DEFAULT_PROGRAM_LEVEL).is_ok());
    }
}
