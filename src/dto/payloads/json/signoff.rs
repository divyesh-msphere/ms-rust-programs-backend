use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ListSignoffsPayload {
    /// The program's ID or its slug.
    pub program: String,
    pub application: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetSignoffPayload {
    pub program: String,
    pub application: String,
    pub signoff: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateSignoffPathPayload {
    pub program: String,
    pub application: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateSignoffBodyPayload {
    /// The approval step you were looking at. If the application has moved on since, the
    /// decision is refused with 409. Omit it to decide whatever step is current.
    pub approval_step_id: Option<String>,
    /// `approved` or `rejected`.
    pub decision: String,
    pub comment: Option<String>,
}
