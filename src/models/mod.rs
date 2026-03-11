pub mod account;
pub mod amount;
pub mod currency;
pub mod journal_entry;
pub mod period;
pub mod report;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct DataResponse<T: Serialize> {
    pub data: T,
}

#[derive(Serialize)]
pub struct ListResponse<T: Serialize> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl PaginationParams {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(200)
    }
}
