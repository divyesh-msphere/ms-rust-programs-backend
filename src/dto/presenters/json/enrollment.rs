use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct BuLocationPresenter {
    pub code: String,
    /// The name of the location; `null` when it was not known at enroll time.
    pub name: Option<String>,
    /// The business unit of the location, `{code, name}`; `null` when none was sent.
    pub bu: Option<BuPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WebsitePresenter {
    pub code: String,
    pub name: Option<String>,
    pub domain: Option<String>,
}

/// The business unit of a store.
#[derive(Debug, Serialize, Clone)]
pub struct BuPresenter {
    pub code: String,
    pub name: Option<String>,
}

/// A store enrolled in a program on one website.
#[derive(Debug, Serialize, Clone)]
pub struct EnrollmentPresenter {
    pub path: String,
    pub id: String,
    pub application_id: Option<String>,
    pub bu_location: BuLocationPresenter,
    pub website: WebsitePresenter,
    pub customer_name: Option<String>,
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
    /// How many stores match the filter (the "N STORES" count).
    pub total_size: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchItemResultPresenter {
    pub bu_location_code: String,
    pub website_code: String,
    /// `created`, `already_enrolled` or `failed`.
    pub outcome: &'static str,
    /// The new store's id when it was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchCreateEnrollmentsPresenter {
    pub created: usize,
    pub results: Vec<BatchItemResultPresenter>,
}
