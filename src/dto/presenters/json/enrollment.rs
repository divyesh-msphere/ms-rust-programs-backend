use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct EnrollmentPresenter {
    pub path: String,
    pub id: String,
    pub application_id: Option<String>,
    pub store_id: String,
    /// `null` = show "Unresolved store".
    pub store_name: Option<String>,
    pub store_resolved: bool,
    pub website_id: String,
    pub website_name: Option<String>,
    pub customer_group: Option<String>,
    pub status: String,
    /// The status changes allowed from here (the Actions dropdown).
    pub actions: Vec<&'static str>,
    pub enrolled_time: Option<String>,
    pub contracts_count: i32,
    pub last_contract_time: Option<String>,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListEnrollmentsPresenter {
    pub results: Vec<EnrollmentPresenter>,
    pub next_page_token: Option<String>,
    /// How many enrollments match the filter (the "N STORES" count).
    pub total_size: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchItemResultPresenter {
    pub store_id: String,
    pub website_id: String,
    /// `created`, `already_enrolled` or `failed`.
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchCreateEnrollmentsPresenter {
    pub created: usize,
    pub results: Vec<BatchItemResultPresenter>,
}
