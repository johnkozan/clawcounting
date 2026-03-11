use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Currency {
    pub id: String,
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub asset_scale: u32,
    pub asset_type: String,
    pub caip19_id: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCurrencyRequest {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub asset_scale: u32,
    pub asset_type: String,
    pub caip19_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCurrencyRequest {
    pub name: Option<String>,
    pub symbol: Option<String>,
}
