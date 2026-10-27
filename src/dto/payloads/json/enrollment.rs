use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ListEnrollmentsPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListEnrollmentsQueryPayload {
    /// The "All websites" filter.
    pub website_id: Option<String>,
    /// Only these statuses, comma-separated (`active,suspended`). Omit for all.
    pub status: Option<String>,
    /// A prefix of the store name (case-insensitive).
    pub store: Option<String>,
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetEnrollmentPayload {
    pub program: String,
    pub enrollment: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEnrollmentBodyPayload {
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub customer_group: Option<String>,
    /// `pending` or `active` (default).
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateEnrollmentPathPayload {
    pub program: String,
    pub enrollment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateEnrollmentBodyPayload {
    pub status: Option<String>,
    /// An empty string clears the group.
    pub customer_group: Option<String>,
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
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub customer_group: Option<String>,
    pub status: Option<String>,
}
