use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

use crate::handlers;
use crate::models;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ClawCounting Accounting API",
        version = "0.1.0",
        description = "Foundational double-entry bookkeeping engine for AI agents. All monetary amounts are i128 integers in the smallest currency unit."
    ),
    paths(
        // Auth
        handlers::auth::login,
        handlers::auth::refresh,
        handlers::auth::me,
        // Currencies
        handlers::currencies::create_currency,
        handlers::currencies::list_currencies,
        handlers::currencies::get_currency,
        handlers::currencies::update_currency,
        // Accounts
        handlers::accounts::create_account,
        handlers::accounts::list_accounts,
        handlers::accounts::get_account,
        handlers::accounts::update_account,
        handlers::accounts::get_sub_accounts,
        handlers::accounts::get_balance,
        handlers::accounts::get_transactions,
        // Journal Entries
        handlers::journal_entries::create_journal_entry,
        handlers::journal_entries::list_journal_entries,
        handlers::journal_entries::get_journal_entry,
        handlers::journal_entries::reverse_journal_entry,
        // Periods
        handlers::periods::create_period,
        handlers::periods::list_periods,
        handlers::periods::get_period,
        handlers::periods::close_period,
        // Reports
        handlers::reports::trial_balance,
        handlers::reports::balance_sheet,
        handlers::reports::income_statement,
        handlers::reports::general_ledger,
        // Settings
        handlers::settings::update_settings,
        handlers::settings::get_settings,
        // Users
        handlers::users::create_user,
        handlers::users::list_users,
        handlers::users::get_user,
        handlers::users::update_user,
        handlers::users::create_service_account,
    ),
    components(
        schemas(
            // Envelopes
            models::DataResponse<models::currency::Currency>,
            models::ListResponse<models::currency::Currency>,
            models::DataResponse<models::account::Account>,
            models::ListResponse<models::account::Account>,
            models::DataResponse<models::journal_entry::JournalEntryWithLines>,
            models::ListResponse<models::journal_entry::JournalEntry>,
            models::ListResponse<models::journal_entry::JournalEntryLine>,
            models::DataResponse<models::period::FinancialPeriod>,
            models::ListResponse<models::period::FinancialPeriod>,
            models::DataResponse<models::period::ClosingResult>,
            models::DataResponse<models::user::UserResponse>,
            models::ListResponse<models::user::UserResponse>,
            models::DataResponse<models::user::ServiceAccountCreatedResponse>,
            models::DataResponse<models::user::TokenResponse>,
            models::DataResponse<models::user::RefreshResponse>,
            models::DataResponse<models::report::TrialBalanceReport>,
            models::DataResponse<models::report::BalanceSheetReport>,
            models::DataResponse<models::report::IncomeStatementReport>,
            models::DataResponse<models::report::GeneralLedgerReport>,
            models::DataResponse<models::journal_entry::BalanceResponse>,
            models::DataResponse<Vec<models::account::Account>>,
            // Domain models
            models::currency::Currency,
            models::currency::CreateCurrencyRequest,
            models::currency::UpdateCurrencyRequest,
            models::account::Account,
            models::account::CreateAccountRequest,
            models::account::UpdateAccountRequest,
            models::journal_entry::JournalEntry,
            models::journal_entry::JournalEntryLine,
            models::journal_entry::JournalEntryWithLines,
            models::journal_entry::CreateJournalEntryRequest,
            models::journal_entry::CreateLineRequest,
            models::journal_entry::ReverseRequest,
            models::journal_entry::BalanceResponse,
            models::period::FinancialPeriod,
            models::period::CreatePeriodRequest,
            models::period::ClosingResult,
            models::user::UserResponse,
            models::user::CreateUserRequest,
            models::user::CreateServiceAccountRequest,
            models::user::UpdateUserRequest,
            models::user::LoginRequest,
            models::user::TokenResponse,
            models::user::RefreshRequest,
            models::user::RefreshResponse,
            models::user::ServiceAccountCreatedResponse,
            // Reports
            models::report::TrialBalanceReport,
            models::report::TrialBalanceRow,
            models::report::BalanceSheetReport,
            models::report::BalanceSheetSection,
            models::report::BalanceSheetRow,
            models::report::IncomeStatementReport,
            models::report::IncomeStatementRow,
            models::report::GeneralLedgerReport,
            models::report::GeneralLedgerLine,
            // Settings
            handlers::settings::UpdateSettingsRequest,
            handlers::settings::SettingsResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication (login, refresh, me)"),
        (name = "Currencies", description = "Currency management"),
        (name = "Accounts", description = "Chart of accounts"),
        (name = "Journal Entries", description = "Double-entry journal entries"),
        (name = "Periods", description = "Financial period management"),
        (name = "Reports", description = "Financial reports"),
        (name = "Settings", description = "Instance settings"),
        (name = "Users", description = "User and service account management"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "JWT access token or API key (tsk_...). Use Authorization: Bearer <token>",
                    ))
                    .build(),
            ),
        );
    }
}
