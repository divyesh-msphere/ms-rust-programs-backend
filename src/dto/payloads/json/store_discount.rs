use serde::{Deserialize, Deserializer};

/// Tells a field that is absent (`None`: leave it) from one that is `null` (`Some(None)`: revert it
/// to the inherited value) from one that has a value (`Some(Some(v))`: set it).
fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

/// What one request changes at one level. A field you leave out is untouched, a field set to
/// `null` goes back to the inherited value, a field with a value is set.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct DiscountPatchPayload {
    #[serde(default, deserialize_with = "double_option")]
    pub enabled: Option<Option<bool>>,
    /// `percentage` or `fixed`.
    #[serde(default, rename = "type", deserialize_with = "double_option")]
    pub discount_type: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub amount: Option<Option<f64>>,
    /// Three uppercase letters, for example `USD`.
    #[serde(default, deserialize_with = "double_option")]
    pub currency_code: Option<Option<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListStoreDiscountsPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListStoreDiscountsQueryPayload {
    /// The "All websites" filter.
    pub website_id: Option<String>,
    /// A prefix of the store name (case-insensitive).
    pub store: Option<String>,
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetStoreDiscountPayload {
    pub program: String,
    pub enrollment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateStoreDiscountPathPayload {
    pub program: String,
    pub enrollment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateStoreDiscountBodyPayload {
    /// The change, or `null` to revert everything to inherited.
    #[serde(default, deserialize_with = "double_option")]
    pub discount: Option<Option<DiscountPatchPayload>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateStoreDiscountsPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateStoreDiscountsBodyPayload {
    /// The enrollment ids of the ticked stores (1 to 500).
    pub ids: Vec<String>,
    /// The change, or `null` to revert everything to inherited.
    #[serde(default, deserialize_with = "double_option")]
    pub discount: Option<Option<DiscountPatchPayload>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListDiscountLevelsPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProgramLevelPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateDomainLevelPathPayload {
    pub program: String,
    pub website_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateDiscountLevelBodyPayload {
    /// The change, or `null` to revert everything to inherited.
    #[serde(default, deserialize_with = "double_option")]
    pub discount: Option<Option<DiscountPatchPayload>>,
}
