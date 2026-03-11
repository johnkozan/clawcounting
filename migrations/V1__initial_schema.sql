-- Settings key-value store
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Seed with defaults
INSERT INTO settings (key, value) VALUES ('instance_name', 'ClawCounting');
INSERT INTO settings (key, value) VALUES ('created_at', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

-- Users (human + service accounts)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT,
    api_key_hash TEXT UNIQUE,
    permissions TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (password_hash IS NOT NULL OR api_key_hash IS NOT NULL)
);

-- Currencies
CREATE TABLE currencies (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    asset_scale INTEGER NOT NULL,
    asset_type TEXT NOT NULL CHECK (asset_type IN ('fiat', 'crypto')),
    caip19_id TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (asset_scale >= 0)
);

-- Accounts (chart of accounts)
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    currency_id TEXT NOT NULL REFERENCES currencies(id),
    account_number TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL CHECK (account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense')),
    normal_balance TEXT NOT NULL CHECK (normal_balance IN ('debit', 'credit')),
    has_subledger INTEGER NOT NULL DEFAULT 0,
    parent_id TEXT REFERENCES accounts(id),
    entity_id TEXT,
    xbrl_tag TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (has_subledger IN (0, 1)),
    CHECK (parent_id IS NULL OR parent_id != id),
    CHECK ((parent_id IS NULL) = (entity_id IS NULL)),
    CHECK (has_subledger = 0 OR parent_id IS NULL)
);

-- Financial periods
CREATE TABLE financial_periods (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    closed_at TEXT,
    closed_by TEXT REFERENCES users(id),
    closing_entry_id TEXT, -- FK added after journal_entries table exists
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (start_date < end_date),
    CHECK ((closed_at IS NULL) = (closed_by IS NULL)),
    CHECK ((closed_at IS NULL) = (closing_entry_id IS NULL))
);

-- Journal entries
CREATE TABLE journal_entries (
    id TEXT PRIMARY KEY,
    period_id TEXT NOT NULL REFERENCES financial_periods(id),
    entry_date TEXT NOT NULL,
    posted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by TEXT NOT NULL REFERENCES users(id),
    description TEXT NOT NULL,
    reference TEXT,
    is_reversal INTEGER NOT NULL DEFAULT 0,
    reverses_id TEXT REFERENCES journal_entries(id),
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (is_reversal IN (0, 1)),
    CHECK ((is_reversal = 1) = (reverses_id IS NOT NULL))
);

CREATE INDEX idx_journal_entries_period ON journal_entries(period_id);
CREATE INDEX idx_journal_entries_date ON journal_entries(entry_date);
CREATE INDEX idx_journal_entries_reverses ON journal_entries(reverses_id) WHERE reverses_id IS NOT NULL;

-- Journal entry lines
CREATE TABLE journal_entry_lines (
    id TEXT PRIMARY KEY,
    journal_entry_id TEXT NOT NULL REFERENCES journal_entries(id),
    account_id TEXT NOT NULL REFERENCES accounts(id),
    debit_amount BLOB NOT NULL,
    credit_amount BLOB NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(debit_amount) = 16),
    CHECK (length(credit_amount) = 16),
    CHECK (debit_amount >= X'80000000000000000000000000000000'),
    CHECK (credit_amount >= X'80000000000000000000000000000000'),
    CHECK (NOT (debit_amount > X'80000000000000000000000000000000'
            AND credit_amount > X'80000000000000000000000000000000'))
);

CREATE INDEX idx_jel_journal_entry ON journal_entry_lines(journal_entry_id);
CREATE INDEX idx_jel_account ON journal_entry_lines(account_id);

-- Per-period materialized balance table
CREATE TABLE account_balances (
    account_id TEXT NOT NULL REFERENCES accounts(id),
    period_id TEXT NOT NULL REFERENCES financial_periods(id),
    total_debits BLOB NOT NULL,
    total_credits BLOB NOT NULL,
    last_updated TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    PRIMARY KEY (account_id, period_id),
    CHECK (length(total_debits) = 16),
    CHECK (length(total_credits) = 16),
    CHECK (total_debits >= X'80000000000000000000000000000000'),
    CHECK (total_credits >= X'80000000000000000000000000000000')
);

CREATE INDEX idx_account_balances_period ON account_balances(period_id);
