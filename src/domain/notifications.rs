// domain/notifications.rs
//
// The Notifications page of a program (Settings): a grid of trigger points x channels. Each trigger
// point says which channels tell the customer when it happens.
use async_trait::async_trait;
use std::collections::HashSet;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// The channels of the grid, in column order. They are the same on every program.
pub const CHANNELS: [&str; 4] = ["email", "sms", "push", "in_app"];

const MAX_TRIGGERS: usize = 100;
const MAX_KEY_LEN: usize = 64;
const MAX_LABEL_LEN: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationTrigger {
    /// Stable machine name, e.g. `balance_threshold`.
    pub key: String,
    /// What the admin sees, e.g. `Balance Threshold`.
    pub label: String,
    pub email: bool,
    pub sms: bool,
    pub push: bool,
    pub in_app: bool,
}

#[derive(Debug, Clone)]
pub struct Notifications {
    pub program_id: Uuid,
    /// The program's `config_version`: send it back with a save to detect a concurrent change.
    pub version: i32,
    pub triggers: Vec<NotificationTrigger>,
}

impl Notifications {
    /// The AEP path of the page.
    pub fn path(&self) -> String {
        format!("programs/{}/notifications", self.program_id)
    }
}

/// The trigger points a program starts with when it picks the module.
pub fn default_triggers() -> Vec<NotificationTrigger> {
    let trigger = |key: &str, label: &str, channels: [bool; 4]| NotificationTrigger {
        key: key.to_string(),
        label: label.to_string(),
        email: channels[0],
        sms: channels[1],
        push: channels[2],
        in_app: channels[3],
    };
    vec![
        trigger("enrollment_approved", "Enrollment Approved", [true, false, false, true]),
        trigger("enrollment_denied", "Enrollment Denied", [true, false, false, true]),
        trigger("credit_limit_change", "Credit Limit Change", [true, false, false, false]),
        trigger("payment_due", "Payment Due", [true, true, true, true]),
        trigger("payment_overdue", "Payment Overdue", [true, true, true, true]),
        trigger("payment_received", "Payment Received", [true, false, false, true]),
        trigger("account_suspended", "Account Suspended", [true, true, true, true]),
        trigger("balance_threshold", "Balance Threshold", [true, false, false, false]),
    ]
}

/// A key is a lower-case machine name: letters, digits and `_`, starting with a letter.
fn valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Checks the whole grid an admin sends before it replaces the saved one. Keys and labels are
/// trimmed; the order of the list is the order of the rows.
pub fn validate_triggers(triggers: Vec<NotificationTrigger>) -> Result<Vec<NotificationTrigger>, ApplicationError> {
    if triggers.len() > MAX_TRIGGERS {
        return Err(ApplicationError::validation_error(&format!("at most {MAX_TRIGGERS} triggers are allowed")));
    }
    let mut seen = HashSet::new();
    let mut checked = Vec::with_capacity(triggers.len());
    for mut trigger in triggers {
        trigger.key = trigger.key.trim().to_string();
        trigger.label = trigger.label.trim().to_string();
        if trigger.key.len() > MAX_KEY_LEN || !valid_key(&trigger.key) {
            return Err(ApplicationError::validation_error(&format!(
                "trigger key {:?} must be lower-case letters, digits and _, start with a letter, and be at most {MAX_KEY_LEN} characters",
                trigger.key
            )));
        }
        if trigger.label.is_empty() || trigger.label.chars().count() > MAX_LABEL_LEN {
            return Err(ApplicationError::validation_error(&format!(
                "trigger {:?} needs a label of 1 to {MAX_LABEL_LEN} characters",
                trigger.key
            )));
        }
        if !seen.insert(trigger.key.clone()) {
            return Err(ApplicationError::validation_error(&format!("trigger key {:?} appears twice", trigger.key)));
        }
        checked.push(trigger);
    }
    Ok(checked)
}

#[async_trait]
pub trait NotificationsRepository: Send + Sync {
    /// The page of a live program that has the module. 404 when it does not.
    async fn get(&self, program_id: Uuid) -> Result<Notifications, ApplicationError>;
    /// Replaces the whole grid: rows missing from `triggers` are removed, new ones added. One
    /// transaction and one version bump; 409 when `expected_version` is stale. Saving the grid as it
    /// already is changes nothing.
    async fn replace(
        &self,
        program_id: Uuid,
        expected_version: Option<i32>,
        triggers: Vec<NotificationTrigger>,
        actor: &str,
    ) -> Result<Notifications, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger(key: &str, label: &str) -> NotificationTrigger {
        NotificationTrigger { key: key.into(), label: label.into(), email: true, sms: false, push: false, in_app: false }
    }

    #[test]
    fn the_defaults_are_valid() {
        let defaults = default_triggers();
        assert_eq!(defaults.len(), 8);
        assert_eq!(validate_triggers(defaults.clone()).unwrap(), defaults);
    }

    #[test]
    fn keys_and_labels_are_trimmed() {
        let checked = validate_triggers(vec![trigger(" balance_threshold ", " Balance Threshold ")]).unwrap();
        assert_eq!(checked[0].key, "balance_threshold");
        assert_eq!(checked[0].label, "Balance Threshold");
    }

    #[test]
    fn bad_keys_are_rejected() {
        for key in ["", "Balance", "1abc", "a-b", "a b"] {
            assert!(validate_triggers(vec![trigger(key, "x")]).is_err(), "{key:?}");
        }
    }

    #[test]
    fn an_empty_label_or_a_repeated_key_is_rejected() {
        assert!(validate_triggers(vec![trigger("a", "  ")]).is_err());
        assert!(validate_triggers(vec![trigger("a", "A"), trigger("a", "B")]).is_err());
    }
}
