pub mod accounts;
pub mod currencies;
pub mod journal_entries;
pub mod periods;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clawcounting", about = "Foundational double-entry bookkeeping engine for AI agents")]
pub struct Cli {
    /// Path to the SQLite database file
    #[arg(long = "db", global = true)]
    pub db_path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP server
    Server,

    /// Manage accounts
    Accounts {
        #[command(subcommand)]
        command: AccountsCommands,
    },

    /// Manage journal entries
    JournalEntries {
        #[command(subcommand)]
        command: JournalEntriesCommands,
    },

    /// Manage financial periods
    Periods {
        #[command(subcommand)]
        command: PeriodsCommands,
    },

    /// Manage currencies
    Currencies {
        #[command(subcommand)]
        command: CurrenciesCommands,
    },

    /// Manage settings
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
    },

    /// Verify balance integrity
    VerifyBalances,

    /// Backup the database
    Backup,

    /// Restore database from backup
    Restore {
        /// Path to the backup file
        backup_file: String,
    },
}

#[derive(Subcommand)]
pub enum AccountsCommands {
    /// List all accounts
    List {
        /// Filter by account type
        #[arg(long = "type")]
        account_type: Option<String>,
        /// Filter by currency ID
        #[arg(long)]
        currency: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create an account
    Create {
        /// Account name
        #[arg(long)]
        name: String,
        /// Currency ID
        #[arg(long)]
        currency: String,
        /// Account type (asset, liability, equity, revenue, expense)
        #[arg(long = "type")]
        account_type: String,
        /// Normal balance (debit or credit)
        #[arg(long = "normal-balance")]
        normal_balance: String,
        /// Account number
        #[arg(long)]
        number: String,
        /// Enable subledger
        #[arg(long)]
        subledger: bool,
        /// Parent account ID (for sub-accounts)
        #[arg(long)]
        parent: Option<String>,
        /// Entity ID (for sub-accounts)
        #[arg(long)]
        entity: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get account details
    Get {
        /// Account ID
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum JournalEntriesCommands {
    /// List journal entries
    List {
        /// Filter by period ID
        #[arg(long)]
        period: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a journal entry from a JSON file
    Create {
        /// Path to JSON file with entry data
        #[arg(long)]
        file: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get journal entry details
    Get {
        /// Journal entry ID
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reverse a journal entry
    Reverse {
        /// Journal entry ID to reverse
        id: String,
        /// Date for the reversal entry (defaults to original date)
        #[arg(long)]
        date: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum PeriodsCommands {
    /// List financial periods
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a financial period
    Create {
        /// Period name
        #[arg(long)]
        name: String,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: String,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Close a financial period
    Close {
        /// Period ID
        id: String,
        /// Preview closing entry without committing
        #[arg(long)]
        preview: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum CurrenciesCommands {
    /// List currencies
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a currency
    Create {
        /// Currency code
        #[arg(long)]
        code: String,
        /// Currency name
        #[arg(long)]
        name: String,
        /// Currency symbol
        #[arg(long)]
        symbol: String,
        /// Number of decimal places
        #[arg(long = "asset-scale")]
        asset_scale: u32,
        /// Asset type (fiat or crypto)
        #[arg(long = "type")]
        asset_type: String,
        /// CAIP-19 identifier
        #[arg(long = "caip19")]
        caip19: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a fiat currency from ISO 4217 code
    CreateFiat {
        /// ISO 4217 currency code (e.g., USD, EUR, GBP)
        code: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get currency details
    Get {
        /// Currency ID
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum SettingsCommands {
    /// Set the retained earnings account
    #[command(name = "set")]
    Set {
        /// Setting key
        key: String,
        /// Setting value
        value: String,
    },
}
