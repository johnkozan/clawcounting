use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCurrencyRequest {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub asset_scale: u32,
    pub asset_type: String,
    pub caip19_id: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCurrencyRequest {
    pub name: Option<String>,
    pub symbol: Option<String>,
}
