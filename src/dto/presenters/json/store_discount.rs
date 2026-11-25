use crate::dto::presenters::json::enrollment::BuPresenter;
use serde::Serialize;

/// What one level sets. A field that is `null` is not set at that level (it is inherited).
#[derive(Debug, Serialize, Clone)]
pub struct DiscountConfigPresenter {
    pub enabled: Option<bool>,
    #[serde(rename = "type")]
    pub discount_type: Option<String>,
    pub amount: Option<f64>,
    pub currency_code: Option<String>,
}

/// The discount a store actually gets.
#[derive(Debug, Serialize, Clone)]
pub struct ResolvedDiscountPresenter {
    pub enabled: bool,
    #[serde(rename = "type")]
    pub discount_type: String,
    pub amount: f64,
    pub currency_code: Option<String>,
}

/// Which level each resolved field came from: `program`, `business_unit`, `website` or `store`.
#[derive(Debug, Serialize, Clone)]
pub struct DiscountSourcesPresenter {
    pub enabled: &'static str,
    #[serde(rename = "type")]
    pub discount_type: &'static str,
    pub amount: &'static str,
    pub currency_code: &'static str,
}

#[derive(Debug, Serialize, Clone)]
pub struct StoreDiscountPresenter {
    pub path: String,
    /// The enrollment id.
    pub id: String,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    /// The business unit of the store (code and name), the parent whose discount the website inherits.
    /// `null` when the store was enrolled without one.
    pub bu: Option<BuPresenter>,
    pub enrollment_status: String,
    /// The Resolved Discount column.
    pub resolved: ResolvedDiscountPresenter,
    /// The badge next to each field in the edit panel.
    pub sources: DiscountSourcesPresenter,
    /// The Set At column: the highest level that set anything.
    pub set_at: &'static str,
    /// "Parent set": at least one field is taken from the business unit, not set on the website.
    pub inherited_from_parent: bool,
    /// What this store sets itself; every field `null` means it inherits everything.
    pub store_override: DiscountConfigPresenter,
    /// What the store would get if its own override were reverted.
    pub inherited: ResolvedDiscountPresenter,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListStoreDiscountsPresenter {
    pub results: Vec<StoreDiscountPresenter>,
    pub next_page_token: Option<String>,
    /// How many enrolled stores match the filters.
    pub total_size: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchStoreDiscountsPresenter {
    /// How many stores really changed.
    pub updated: usize,
    pub results: Vec<StoreDiscountPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WebsiteLevelPresenter {
    pub website_id: String,
    pub discount: DiscountConfigPresenter,
}

#[derive(Debug, Serialize, Clone)]
pub struct BusinessUnitLevelPresenter {
    pub bu_unit_code: String,
    /// From the stores enrolled with this business unit.
    pub name: Option<String>,
    pub discount: DiscountConfigPresenter,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiscountLevelsPresenter {
    pub program: DiscountConfigPresenter,
    /// The parents: a business unit that sets something is inherited by its websites.
    pub business_units: Vec<BusinessUnitLevelPresenter>,
    /// The children that set something of their own.
    pub websites: Vec<WebsiteLevelPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiscountLevelPresenter {
    /// `program`, `business_unit` or `website`.
    pub level: &'static str,
    pub website_id: Option<String>,
    pub bu_unit_code: Option<String>,
    /// What that level sets afterwards.
    pub discount: DiscountConfigPresenter,
}
