// domain/store_discount.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// The module a program must have switched on to use store discounts.
pub const MODULE_KEY: &str = "catalog_pricing.store_discounts.control_panel";

pub const PERCENTAGE: &str = "percentage";
pub const FIXED: &str = "fixed";

/// The four fields of a discount, each inherited on its own.
pub const FIELDS: [&str; 4] = ["enabled", "type", "amount", "currency_code"];

/// How many stores one mass edit may carry.
pub const MAX_BATCH: usize = 500;

const MAX_AMOUNT: f64 = 1_000_000.0;

/// Where a value comes from. Priority: store over domain over program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    Program,
    Domain,
    Store,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Program => "program",
            Level::Domain => "domain",
            Level::Store => "store",
        }
    }
}

/// What ONE level sets. Every field is optional: a field left unset is inherited from the level
/// above it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DiscountConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub discount_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
}

impl DiscountConfig {
    /// A level's stored JSON; anything unreadable counts as "sets nothing".
    pub fn from_json(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// A change to one level. Per field: `None` leaves it, `Some(None)` reverts it to the inherited
/// value, `Some(Some(v))` sets it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiscountPatch {
    pub enabled: Option<Option<bool>>,
    pub discount_type: Option<Option<String>>,
    pub amount: Option<Option<f64>>,
    pub currency_code: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiscountChange {
    /// "Revert all to inherited": the level sets nothing.
    RevertAll,
    Patch(DiscountPatch),
}

impl DiscountChange {
    /// What a level looks like after the change.
    pub fn apply(&self, existing: &DiscountConfig) -> DiscountConfig {
        match self {
            DiscountChange::RevertAll => DiscountConfig::default(),
            DiscountChange::Patch(patch) => DiscountConfig {
                enabled: patch.enabled.unwrap_or(existing.enabled),
                discount_type: patch.discount_type.clone().unwrap_or_else(|| existing.discount_type.clone()),
                amount: patch.amount.unwrap_or(existing.amount),
                currency_code: patch.currency_code.clone().unwrap_or_else(|| existing.currency_code.clone()),
            },
        }
    }
}

/// The discount a store actually gets: every field taken from the highest level that set it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resolved {
    pub enabled: bool,
    pub discount_type: String,
    pub amount: f64,
    pub currency_code: Option<String>,
}

/// Which level each resolved field came from (a field no level set reads "program").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sources {
    pub enabled: Level,
    pub discount_type: Level,
    pub amount: Level,
    pub currency_code: Level,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Resolution {
    pub resolved: Resolved,
    pub sources: Sources,
    /// The highest level that set anything (the Set At badge); `Program` when nothing overrides it.
    pub set_at: Level,
}

fn pick<T: Clone>(program: Option<T>, domain: Option<T>, store: Option<T>) -> (Option<T>, Level) {
    if store.is_some() {
        (store, Level::Store)
    } else if domain.is_some() {
        (domain, Level::Domain)
    } else {
        (program, Level::Program)
    }
}

/// Program -> Domain -> Store, field by field. With nothing set anywhere the discount is disabled,
/// a percentage, of zero.
pub fn resolve(program: &DiscountConfig, domain: &DiscountConfig, store: &DiscountConfig) -> Resolution {
    let (enabled, enabled_at) = pick(program.enabled, domain.enabled, store.enabled);
    let (discount_type, type_at) = pick(program.discount_type.clone(), domain.discount_type.clone(), store.discount_type.clone());
    let (amount, amount_at) = pick(program.amount, domain.amount, store.amount);
    let (currency_code, currency_at) =
        pick(program.currency_code.clone(), domain.currency_code.clone(), store.currency_code.clone());

    let sources = Sources { enabled: enabled_at, discount_type: type_at, amount: amount_at, currency_code: currency_at };
    let set_at = [sources.enabled, sources.discount_type, sources.amount, sources.currency_code]
        .into_iter()
        .max()
        .unwrap_or(Level::Program);
    Resolution {
        resolved: Resolved {
            enabled: enabled.unwrap_or(false),
            discount_type: discount_type.unwrap_or_else(|| PERCENTAGE.to_string()),
            amount: amount.unwrap_or(0.0),
            currency_code,
        },
        sources,
        set_at,
    }
}

/// One level on its own: each value must be valid.
pub fn validate_config(config: &DiscountConfig) -> Result<(), ApplicationError> {
    if let Some(kind) = &config.discount_type
        && kind != PERCENTAGE
        && kind != FIXED
    {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "type {kind:?} must be {PERCENTAGE:?} or {FIXED:?}"
        )));
    }
    if let Some(amount) = config.amount {
        if !amount.is_finite() || !(0.0..=MAX_AMOUNT).contains(&amount) {
            return Err(ApplicationError::unprocessable_entity(&format!("amount must be between 0 and {MAX_AMOUNT}")));
        }
        // Money has cents, no more.
        let cents = amount * 100.0;
        if (cents - cents.round()).abs() > 1e-6 {
            return Err(ApplicationError::unprocessable_entity("amount can have at most 2 decimal places"));
        }
    }
    if let Some(code) = &config.currency_code
        && !(code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()))
    {
        return Err(ApplicationError::unprocessable_entity("currency_code must be 3 uppercase letters, for example USD"));
    }
    Ok(())
}

/// The combination a store would end up with must make sense: a percentage cannot pass 100.
pub fn validate_resolved(resolved: &Resolved) -> Result<(), ApplicationError> {
    if resolved.discount_type == PERCENTAGE && resolved.amount > 100.0 {
        return Err(ApplicationError::unprocessable_entity(
            "a percentage discount cannot be more than 100 (change the amount, or the type to fixed)",
        ));
    }
    Ok(())
}

/// A store (enrollment) as the Store Discounts page lists it, with its discount worked out.
#[derive(Debug, Clone)]
pub struct StoreDiscountRow {
    pub enrollment_id: Uuid,
    pub program_id: Uuid,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub enrollment_status: String,
    pub resolved: Resolved,
    pub sources: Sources,
    pub set_at: Level,
    /// What this store sets itself (empty = it inherits everything).
    pub store_override: DiscountConfig,
    /// What it would get if its own override were reverted (what "revert" drops back to).
    pub inherited: Resolved,
}

impl StoreDiscountRow {
    pub fn build(
        enrollment_id: Uuid,
        program_id: Uuid,
        store: (String, Option<String>, String, Option<String>, String),
        program: &DiscountConfig,
        domain: &DiscountConfig,
        store_override: DiscountConfig,
    ) -> Self {
        let (store_id, store_name, website_id, website_name, enrollment_status) = store;
        let resolution = resolve(program, domain, &store_override);
        let inherited = resolve(program, domain, &DiscountConfig::default()).resolved;
        Self {
            enrollment_id,
            program_id,
            store_id,
            store_name,
            website_id,
            website_name,
            enrollment_status,
            resolved: resolution.resolved,
            sources: resolution.sources,
            set_at: resolution.set_at,
            store_override,
            inherited,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StoreDiscountFilter {
    pub website_id: Option<String>,
    /// A prefix of the store name (case-insensitive).
    pub store_prefix: Option<String>,
}

/// The two levels above a store.
#[derive(Debug, Clone, PartialEq)]
pub enum LevelTarget {
    Program,
    Domain(String),
}

#[derive(Debug, Clone)]
pub struct DiscountLevels {
    pub program: DiscountConfig,
    /// Websites that set something of their own.
    pub domains: Vec<(String, DiscountConfig)>,
}

pub fn validate_website_id(website_id: &str) -> Result<(), ApplicationError> {
    if website_id.trim().is_empty() || website_id.chars().count() > 64 {
        return Err(ApplicationError::unprocessable_entity("website_id must be 1-64 characters"));
    }
    Ok(())
}

#[async_trait]
pub trait StoreDiscountRepository: Send + Sync {
    /// Enrolled stores (pending, active or suspended), newest first, with the discount resolved for
    /// the page only. Also the number matching the filter. The program must have the Store
    /// Discounts module switched on (`failed_precondition` otherwise).
    async fn list(
        &self,
        program_id: Uuid,
        filter: &StoreDiscountFilter,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<StoreDiscountRow>, Option<Uuid>, i64), ApplicationError>;

    async fn get(&self, program_id: Uuid, enrollment_id: Uuid) -> Result<StoreDiscountRow, ApplicationError>;

    /// Changes the store level of ONE store: one transaction with the program's version bump, the
    /// audit entry and the outbox event. A change that alters nothing writes nothing.
    async fn update_store(
        &self,
        program_id: Uuid,
        enrollment_id: Uuid,
        change: &DiscountChange,
        actor: &str,
    ) -> Result<StoreDiscountRow, ApplicationError>;

    /// The same change for many stores with one write statement, ONE version bump, ONE audit entry
    /// and ONE outbox event, whatever the number. Every id must be an enrolled store of the
    /// program (`not_found` otherwise) and the whole batch is refused if one result is invalid.
    /// Returns how many stores really changed, and every store's row.
    async fn batch_update_stores(
        &self,
        program_id: Uuid,
        enrollment_ids: &[Uuid],
        change: &DiscountChange,
        actor: &str,
    ) -> Result<(usize, Vec<StoreDiscountRow>), ApplicationError>;

    /// The program level and every website that sets something.
    async fn levels(&self, program_id: Uuid) -> Result<DiscountLevels, ApplicationError>;

    /// Changes the program level or one website's level. Returns what that level sets afterwards.
    async fn update_level(
        &self,
        program_id: Uuid,
        target: &LevelTarget,
        change: &DiscountChange,
        actor: &str,
    ) -> Result<DiscountConfig, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cfg(enabled: Option<bool>, kind: Option<&str>, amount: Option<f64>) -> DiscountConfig {
        DiscountConfig { enabled, discount_type: kind.map(String::from), amount, currency_code: None }
    }

    #[test]
    fn nothing_set_means_disabled_percentage_of_zero_from_program() {
        let r = resolve(&DiscountConfig::default(), &DiscountConfig::default(), &DiscountConfig::default());
        assert_eq!(r.resolved, Resolved { enabled: false, discount_type: "percentage".into(), amount: 0.0, currency_code: None });
        assert_eq!(r.set_at, Level::Program);
        assert_eq!(r.sources.amount, Level::Program);
    }

    #[test]
    fn each_field_is_taken_from_the_highest_level_that_set_it() {
        let program = cfg(Some(true), Some("percentage"), Some(5.0));
        let domain = cfg(None, None, Some(10.0));
        let store = cfg(None, Some("fixed"), None);
        let r = resolve(&program, &domain, &store);
        assert!(r.resolved.enabled);
        assert_eq!(r.resolved.discount_type, "fixed");
        assert_eq!(r.resolved.amount, 10.0);
        assert_eq!(r.sources.enabled, Level::Program);
        assert_eq!(r.sources.discount_type, Level::Store);
        assert_eq!(r.sources.amount, Level::Domain);
        assert_eq!(r.set_at, Level::Store, "the badge is the highest level that set anything");
    }

    #[test]
    fn set_at_is_domain_when_only_the_domain_overrides() {
        let r = resolve(&cfg(Some(true), None, Some(5.0)), &cfg(None, None, Some(8.0)), &DiscountConfig::default());
        assert_eq!(r.set_at, Level::Domain);
    }

    #[test]
    fn a_store_can_switch_a_program_discount_off() {
        let r = resolve(&cfg(Some(true), None, Some(5.0)), &DiscountConfig::default(), &cfg(Some(false), None, None));
        assert!(!r.resolved.enabled);
        assert_eq!(r.sources.enabled, Level::Store);
        assert_eq!(r.resolved.amount, 5.0);
    }

    #[test]
    fn patch_sets_reverts_and_leaves_fields() {
        let existing = DiscountConfig { enabled: Some(true), discount_type: Some("fixed".into()), amount: Some(15.0), currency_code: Some("USD".into()) };
        let patch = DiscountPatch {
            amount: Some(Some(20.0)),
            discount_type: Some(None),
            ..Default::default()
        };
        let out = DiscountChange::Patch(patch).apply(&existing);
        assert_eq!(out.enabled, Some(true), "left alone");
        assert_eq!(out.discount_type, None, "reverted");
        assert_eq!(out.amount, Some(20.0), "set");
        assert_eq!(out.currency_code.as_deref(), Some("USD"));
        assert!(DiscountChange::RevertAll.apply(&existing).is_empty());
        assert_eq!(DiscountChange::Patch(DiscountPatch::default()).apply(&existing), existing);
    }

    #[test]
    fn config_round_trips_through_json_and_ignores_junk() {
        let c = DiscountConfig { enabled: Some(true), discount_type: Some("fixed".into()), amount: Some(11.21), currency_code: Some("GBP".into()) };
        assert_eq!(c.to_json(), json!({"enabled": true, "type": "fixed", "amount": 11.21, "currency_code": "GBP"}));
        assert_eq!(DiscountConfig::from_json(&c.to_json()), c);
        assert!(DiscountConfig::from_json(&json!({})).is_empty());
        assert!(DiscountConfig::from_json(&json!("junk")).is_empty());
        assert!(DiscountConfig::from_json(&json!({"enabled": "yes"})).is_empty());
    }

    #[test]
    fn values_are_validated() {
        assert!(validate_config(&cfg(Some(true), Some("percentage"), Some(15.5))).is_ok());
        assert!(validate_config(&cfg(None, Some("fixed"), Some(11.21))).is_ok());
        assert!(validate_config(&cfg(None, Some("bogus"), None)).is_err());
        assert!(validate_config(&cfg(None, None, Some(-1.0))).is_err());
        assert!(validate_config(&cfg(None, None, Some(1_000_001.0))).is_err());
        assert!(validate_config(&cfg(None, None, Some(1.005))).is_err(), "more than 2 decimals");
        assert!(validate_config(&cfg(None, None, Some(f64::NAN))).is_err());
        let mut c = DiscountConfig::default();
        c.currency_code = Some("usd".into());
        assert!(validate_config(&c).is_err());
        c.currency_code = Some("USD".into());
        assert!(validate_config(&c).is_ok());
        c.currency_code = Some("US".into());
        assert!(validate_config(&c).is_err());
    }

    #[test]
    fn a_percentage_cannot_pass_100_but_a_fixed_amount_can() {
        let pct = Resolved { enabled: true, discount_type: "percentage".into(), amount: 100.0, currency_code: None };
        assert!(validate_resolved(&pct).is_ok());
        assert!(validate_resolved(&Resolved { amount: 100.5, ..pct.clone() }).is_err());
        assert!(validate_resolved(&Resolved { discount_type: "fixed".into(), amount: 250.0, ..pct }).is_ok());
    }

    #[test]
    fn row_shows_what_revert_would_drop_back_to() {
        let program = cfg(Some(true), Some("percentage"), Some(5.0));
        let store = cfg(None, None, Some(9.0));
        let row = StoreDiscountRow::build(
            Uuid::nil(),
            Uuid::nil(),
            ("s".into(), None, "w".into(), None, "active".into()),
            &program,
            &DiscountConfig::default(),
            store,
        );
        assert_eq!(row.resolved.amount, 9.0);
        assert_eq!(row.inherited.amount, 5.0);
        assert_eq!(row.set_at, Level::Store);
    }

    #[test]
    fn website_id_is_bounded() {
        assert!(validate_website_id("cprparts").is_ok());
        assert!(validate_website_id(" ").is_err());
        assert!(validate_website_id(&"w".repeat(65)).is_err());
    }
}
