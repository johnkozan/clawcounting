pub mod account;
pub mod amount;
pub mod currency;
pub mod journal_entry;
pub mod period;
pub mod report;
pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Serialize, utoipa::ToSchema)]
#[schema(bound = "T: utoipa::ToSchema")]
pub struct DataResponse<T: Serialize> {
    pub data: T,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(bound = "T: utoipa::ToSchema")]
pub struct ListResponse<T: Serialize> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl PaginationParams {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(50).min(200)
    }
}

// Type aliases for OpenAPI schema references (used in utoipa::path response annotations)
pub type DataResponseCurrency = DataResponse<currency::Currency>;
pub type DataResponseAccount = DataResponse<account::Account>;
pub type DataResponseJournalEntryWithLines = DataResponse<journal_entry::JournalEntryWithLines>;
pub type DataResponseFinancialPeriod = DataResponse<period::FinancialPeriod>;
pub type DataResponseClosingResult = DataResponse<period::ClosingResult>;
pub type DataResponseUserResponse = DataResponse<user::UserResponse>;
pub type DataResponseServiceAccountCreatedResponse = DataResponse<user::ServiceAccountCreatedResponse>;
pub type DataResponseTokenResponse = DataResponse<user::TokenResponse>;
pub type DataResponseRefreshResponse = DataResponse<user::RefreshResponse>;
pub type DataResponseTrialBalanceReport = DataResponse<report::TrialBalanceReport>;
pub type DataResponseBalanceSheetReport = DataResponse<report::BalanceSheetReport>;
pub type DataResponseIncomeStatementReport = DataResponse<report::IncomeStatementReport>;
pub type DataResponseGeneralLedgerReport = DataResponse<report::GeneralLedgerReport>;
pub type DataResponseBalanceResponse = DataResponse<journal_entry::BalanceResponse>;
pub type DataResponseVecAccount = DataResponse<Vec<account::Account>>;

pub type ListResponseCurrency = ListResponse<currency::Currency>;
pub type ListResponseAccount = ListResponse<account::Account>;
pub type ListResponseJournalEntry = ListResponse<journal_entry::JournalEntry>;
pub type ListResponseJournalEntryLine = ListResponse<journal_entry::JournalEntryLine>;
pub type ListResponseFinancialPeriod = ListResponse<period::FinancialPeriod>;
pub type ListResponseUserResponse = ListResponse<user::UserResponse>;
