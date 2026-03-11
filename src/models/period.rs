use serde::{Deserialize, Serialize};

use super::journal_entry::JournalEntryWithLines;

#[derive(Debug, Clone, Serialize)]
pub struct FinancialPeriod {
    pub id: String,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub closed_at: Option<String>,
    pub closed_by: Option<String>,
    pub closing_entry_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePeriodRequest {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize)]
pub struct ClosingResult {
    pub period: FinancialPeriod,
    pub closing_entry: JournalEntryWithLines,
    pub preview: bool,
}

#[derive(Debug, Deserialize)]
pub struct CloseQuery {
    #[serde(default)]
    pub preview: bool,
}
