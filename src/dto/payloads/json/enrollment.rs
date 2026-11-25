use serde::{Deserialize, Deserializer};

/// Tells an absent field (`None`: leave it) from a `null` one (`Some(None)`: clear it).
fn present_or_null<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListEnrollmentsPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListEnrollmentsQueryPayload {
    /// The search box: bare text (matches the BU location code, website or customer name), or
    /// `bu_location=abc` / `website=abc` / `customer_name=abc` for one field.
    pub search: Option<String>,
    /// Operators and lists, e.g. `bu_location.starts_with=BU,website=[w1,w2],status!=ended`. Text fields
    /// (bu_location, website, customer_name) take an operator: contains (default), not_contains, equals,
    /// not_equals, starts_with, ends_with, is_empty, is_not_empty (customer_name only).
    pub filter: Option<String>,
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetEnrollmentPayload {
    pub program: String,
    /// The store's id.
    pub store: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEnrollmentPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEnrollmentQueryPayload {
    /// Optional client-specified ID (AEP-133); server generates one if omitted.
    pub id: Option<String>,
}

/// The BU location a store belongs to.
#[derive(Debug, Deserialize, Clone)]
pub struct BuLocationPayload {
    /// Its unique code (required).
    pub code: String,
    /// The name of the location (optional; an empty string is the same as none).
    pub name: Option<String>,
    /// The business unit of the location: `{code, name}` (optional).
    pub bu: Option<BuPayload>,
}

/// The website the store is enrolled on.
#[derive(Debug, Deserialize, Clone)]
pub struct WebsitePayload {
    /// Its unique code (required).
    pub code: String,
    pub name: Option<String>,
    pub domain: Option<String>,
}

/// The business unit of the store. Its code is also the parent of the website in the store discounts.
#[derive(Debug, Deserialize, Clone)]
pub struct BuPayload {
    /// Its code (required when `bu` is sent).
    pub code: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEnrollmentBodyPayload {
    pub bu_location: BuLocationPayload,
    pub website: WebsitePayload,
    /// Optional.
    pub customer_name: Option<String>,
    /// `pending` or `active` (default).
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateEnrollmentPathPayload {
    pub program: String,
    /// The store's id.
    pub store: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateEnrollmentBodyPayload {
    /// Any of `pending`, `active`, `suspended`, `ended`.
    pub status: Option<String>,
    /// An empty string clears the customer name.
    pub customer_name: Option<String>,
    /// Only `bu_location.bu` is read: it replaces the business unit, `null` clears it. The code of the
    /// location itself cannot change.
    pub bu_location: Option<UpdateBuLocationPayload>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UpdateBuLocationPayload {
    /// Replaces the name of the location; an empty string clears it.
    pub name: Option<String>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub bu: Option<Option<BuPayload>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchCreateEnrollmentsPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchCreateEnrollmentsBodyPayload {
    pub items: Vec<BatchItemPayload>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchItemPayload {
    pub bu_location: BuLocationPayload,
    pub website: WebsitePayload,
    pub customer_name: Option<String>,
    pub status: Option<String>,
}
