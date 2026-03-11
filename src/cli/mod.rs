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
    List,
}

#[derive(Subcommand)]
pub enum JournalEntriesCommands {
    /// List journal entries
    List,
}

#[derive(Subcommand)]
pub enum PeriodsCommands {
    /// List financial periods
    List,
}

#[derive(Subcommand)]
pub enum CurrenciesCommands {
    /// List currencies
    List,
}
