use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct CreatePublicationPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListPublicationsPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListPublicationsQueryPayload {
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetPublicationPayload {
    pub program: String,
    pub publication: String,
}
