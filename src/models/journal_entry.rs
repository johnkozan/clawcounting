use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct JournalEntry {
    pub id: String,
    pub period_id: String,
    pub entry_date: String,
    pub posted_at: String,
    pub created_by: String,
    pub description: String,
    pub reference: Option<String>,
    pub is_reversal: bool,
    pub reverses_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct JournalEntryLine {
    pub id: String,
    pub journal_entry_id: String,
    pub account_id: String,
    /// Raw i128 amount as string
    pub debit_amount: String,
    /// Raw i128 amount as string
    pub credit_amount: String,
    /// Formatted with decimal places
    pub display_debit: String,
    /// Formatted with decimal places
    pub display_credit: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct JournalEntryWithLines {
    #[serde(flatten)]
    pub entry: JournalEntry,
    pub lines: Vec<JournalEntryLine>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateJournalEntryRequest {
    pub entry_date: String,
    pub description: String,
    pub reference: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub lines: Vec<CreateLineRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateLineRequest {
    pub account_id: String,
    #[serde(default)]
    pub debit_amount: Option<String>,
    #[serde(default)]
    pub credit_amount: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct JournalEntryFilters {
    pub period_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub account_id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl JournalEntryFilters {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(200)
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ReverseRequest {
    pub entry_date: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BalanceResponse {
    pub account_id: String,
    pub period_id: Option<String>,
    pub total_debits: String,
    pub total_credits: String,
    pub net_balance: String,
    pub display_debits: String,
    pub display_credits: String,
    pub display_balance: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct BalanceQuery {
    pub period_id: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TransactionFilters {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl TransactionFilters {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(200)
    }
}
