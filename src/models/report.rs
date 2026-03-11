use serde::{Deserialize, Serialize};

// ── Trial Balance ─────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TrialBalanceRow {
    pub account_id: String,
    pub account_number: String,
    pub account_name: String,
    pub account_type: String,
    pub debit_total: String,
    pub credit_total: String,
    pub display_debit_total: String,
    pub display_credit_total: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TrialBalanceReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_id: Option<String>,
    pub rows: Vec<TrialBalanceRow>,
    pub grand_total_debits: String,
    pub grand_total_credits: String,
    pub display_grand_total_debits: String,
    pub display_grand_total_credits: String,
    pub is_balanced: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TrialBalanceQuery {
    pub period_id: Option<String>,
    pub currency_id: Option<String>,
}

// ── Balance Sheet ─────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BalanceSheetRow {
    pub account_id: String,
    pub account_number: String,
    pub account_name: String,
    pub net_balance: String,
    pub display_balance: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BalanceSheetSection {
    pub label: String,
    pub accounts: Vec<BalanceSheetRow>,
    pub total: String,
    pub display_total: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BalanceSheetReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_id: Option<String>,
    pub assets: BalanceSheetSection,
    pub liabilities: BalanceSheetSection,
    pub equity: BalanceSheetSection,
    pub total_assets: String,
    pub total_liabilities_and_equity: String,
    pub display_total_assets: String,
    pub display_total_liabilities_and_equity: String,
    pub is_balanced: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct BalanceSheetQuery {
    pub period_id: Option<String>,
    pub as_of_date: Option<String>,
}

// ── Income Statement ──────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IncomeStatementRow {
    pub account_id: String,
    pub account_number: String,
    pub account_name: String,
    pub net_amount: String,
    pub display_amount: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IncomeStatementReport {
    pub period_id: String,
    pub revenue: Vec<IncomeStatementRow>,
    pub expenses: Vec<IncomeStatementRow>,
    pub total_revenue: String,
    pub total_expenses: String,
    pub net_income: String,
    pub display_total_revenue: String,
    pub display_total_expenses: String,
    pub display_net_income: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct IncomeStatementQuery {
    pub period_id: Option<String>,
}

// ── General Ledger ────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GeneralLedgerLine {
    pub line_id: String,
    pub journal_entry_id: String,
    pub entry_date: String,
    pub description: String,
    pub reference: Option<String>,
    pub debit_amount: String,
    pub credit_amount: String,
    pub display_debit: String,
    pub display_credit: String,
    pub running_balance: String,
    pub display_running_balance: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GeneralLedgerReport {
    pub account_id: String,
    pub account_number: String,
    pub account_name: String,
    pub normal_balance: String,
    pub starting_balance: String,
    pub display_starting_balance: String,
    pub lines: Vec<GeneralLedgerLine>,
    pub ending_balance: String,
    pub display_ending_balance: String,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct GeneralLedgerQuery {
    pub account_id: Option<String>,
    pub period_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl GeneralLedgerQuery {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(200)
    }

    pub fn is_ascending(&self) -> bool {
        self.sort.as_deref() == Some("asc")
    }
}
