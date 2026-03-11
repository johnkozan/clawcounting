use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Account {
    pub id: String,
    pub currency_id: String,
    pub account_number: String,
    pub name: String,
    pub account_type: String,
    pub normal_balance: String,
    pub has_subledger: bool,
    pub parent_id: Option<String>,
    pub entity_id: Option<String>,
    pub xbrl_tag: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAccountRequest {
    pub currency_id: Option<String>,
    pub account_number: String,
    pub name: String,
    pub account_type: Option<String>,
    pub normal_balance: Option<String>,
    #[serde(default)]
    pub has_subledger: bool,
    pub parent_id: Option<String>,
    pub entity_id: Option<String>,
    pub xbrl_tag: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub xbrl_tag: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct AccountFilters {
    pub account_type: Option<String>,
    pub currency_id: Option<String>,
    pub is_active: Option<bool>,
    pub parent_id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl AccountFilters {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(200)
    }
}
