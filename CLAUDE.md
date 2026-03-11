# ClawCounting Accounting

Foundational double-entry bookkeeping engine for AI agents. Single Rust binary + single SQLite database.

## Tech Stack

- **Backend**: Rust + Axum + rusqlite (bundled, i128_blob) + deadpool-sqlite
- **Database**: SQLite 3 (WAL mode, single file)
- **Migrations**: refinery (embedded .sql, forward-only)
- **Frontend**: SvelteKit SPA (adapter-static) + Tailwind CSS + shadcn-svelte (in `frontend/`)
- **API Docs**: utoipa + swagger-ui (auto-generated OpenAPI)
- **Auth**: API keys (agents) + JWT (web users), permission-based
- **CLI**: clap subcommands (`clawcounting server`, `clawcounting accounts list`, etc.)

## Architecture

- Single-tenant. No org_id anywhere. Multi-tenant = separate instances.
- Server and CLI share the same `services/` layer — identical validation and business logic.
- Server: deadpool-sqlite write pool (size 1) + read pool (size 2-4). CLI: single direct rusqlite connection.
- All monetary amounts stored as i128 BLOBs (16-byte big-endian, MSB-flipped for memcmp ordering).
- Custom SQLite functions: `sum_i128()`, `i128_add()`, `i128_to_text()` — registered on every connection before migrations run.
- Balance updates via `update_balance_on_insert` trigger on `journal_entry_lines` — automatic, within same transaction.
- `BEGIN IMMEDIATE` for all write transactions (journal entries, period close) to prevent race conditions.

## Project Structure

```
src/
  main.rs              # Entrypoint, clap dispatch
  config.rs            # DB path, port, env vars (CLAWCOUNTING_*)
  app_state.rs         # Shared state (connection pools)
  error.rs             # AppError, RFC 7807 responses
  router.rs            # Route assembly
  db/
    connection.rs      # SQLite setup (WAL, pragmas, custom functions, busy_timeout)
    i128_funcs.rs      # Custom SQLite functions
    pool.rs            # deadpool-sqlite pools
    migrations.rs      # refinery embedded migrations
  middleware/auth.rs   # API key + JWT validation
  models/              # Request/response structs (currency, account, journal_entry, period, report)
  handlers/            # Axum route handlers (auth, currencies, accounts, journal_entries, periods, reports)
  services/            # Business logic (currency, account, journal, period, report, balance services)
  cli/                 # CLI command handlers (accounts, journal_entries, periods, currencies, admin)
migrations/
  001_initial_schema.sql
  002_balance_triggers.sql
skill/                 # Agent Skill (agentskills.io standard)
frontend/              # SvelteKit SPA
tests/                 # Integration tests
```

## Key Domain Rules

- **Double-entry**: Every journal entry must balance (total debits == total credits). Minimum 2 lines. All lines same currency.
- **Immutability**: Journal entries and lines are append-only. No UPDATE/DELETE. Corrections via reversing entries only.
- **Periods**: Financial periods have no overlap, no reopen (closed = permanent). Closing generates a closing journal entry that zeroes revenue/expense into retained earnings. Requires `retained_earnings_account_id` in settings.
- **Subledgers**: `has_subledger` flag on accounts. Control accounts can't be posted to directly — use sub-accounts (parent_id + entity_id). Control account balance = sum of children.
- **Currencies**: CAIP-19 identifiers for all currencies. Fiat: `swift:0/iso4217:USD`. Crypto: `eip155:1/slip44:60`. Asset scale defines decimal places.
- **Amount handling**: i128 integers in smallest currency unit (cents, satoshis, wei). Displayed via `display_amount` in API responses. JSON serializes as decimal strings.

## API Conventions

- All endpoints under `/api/v1/` (no org scoping)
- Response envelope: `{ "data": ... }` for single, `{ "data": [...], "has_more": bool, "next_cursor": "..." }` for lists
- Cursor-based pagination (Stripe-style). Default limit 50, max 200.
- Errors: RFC 7807 with `code`, `message`, `field`, `suggestion`
- CLI output: human-readable tables by default, `--json` for machine-readable

## Database

- IDs: UUIDv7 (text primary keys)
- Dates: ISO 8601 strings
- Booleans: INTEGER (0/1)
- Amounts: 16-byte i128 BLOBs
- JSON fields: TEXT with JSON content (permissions, metadata)
- Pragmas: `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000`

## Startup Flow

1. Open raw rusqlite connection
2. Set pragmas
3. Register custom SQLite functions (must happen before migrations — triggers reference them)
4. Run refinery migrations
5. Branch: server creates pools (with post_create hooks for pragmas + functions), CLI keeps direct connection

## Testing

- Integration tests in `tests/` — each file is a separate test binary
- Shared helpers in `tests/common/mod.rs` (setup_db, test fixtures, balance assertions)
- In-memory SQLite (`:memory:`) for tests — no Docker needed
- `cargo test` to run all tests
- Always verify `account_balances` consistency after journal entry tests

## Environment

- `CLAWCOUNTING_DB` — database file path (default: `./clawcounting.db`)
- `CLAWCOUNTING_JWT_SECRET` — JWT signing secret (required for server mode)
- `.env` file loaded via dotenvy (`.env.example` shipped, `.env` gitignored)

## Commands

```
cargo build                    # build
cargo test                     # run all tests
cargo run -- server            # start HTTP server
cargo run -- accounts list     # CLI usage
```
