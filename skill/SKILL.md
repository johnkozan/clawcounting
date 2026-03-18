---
name: clawcounting
description: >
  Double-entry bookkeeping engine for AI agents. Use when performing any
  accounting task: setting up a chart of accounts, defining currencies (fiat
  and crypto), creating financial periods, posting journal entries, querying
  account balances, generating financial reports (trial balance, balance sheet,
  income statement, general ledger), closing fiscal periods, or reversing
  entries. Supports fiat currencies (ISO 4217) and crypto tokens with full
  wei-precision via i128 integers. Interact via CLI (clawcounting ... --json) or
  REST API. Use this skill even if the user says "bookkeeping", "ledger",
  "debit/credit", "chart of accounts", "fiscal year", or "closing entries".
license: MIT
compatibility: Requires the clawcounting binary (Rust). Install via cargo install --git https://github.com/johnkozan/clawcounting or download from GitHub Releases.
metadata:
  author: clawcounting
  version: "0.1"
  openclaw:
    requires:
      bins: ["clawcounting"]
allowed-tools: Bash(clawcounting:*) Bash(curl:*)
---

# ClawCounting Accounting

ClawCounting is a double-entry bookkeeping engine designed for AI agents. Single Rust binary, single SQLite database. All accounting logic is enforced — balanced entries, immutable journals, period controls — so you can post with confidence.

## Installation

```bash
# From source (requires Rust toolchain)
cargo install --git https://github.com/johnkozan/clawcounting

# Verify
clawcounting --version
```

## Interfaces

**CLI** (recommended for agents):
```bash
clawcounting <command> [--json]    # --json for machine-readable output
```
The CLI connects directly to the SQLite database. No server needed. Set `CLAWCOUNTING_DB` to control the database path (default: `./clawcounting.db`).

Commands that create accounting records (journal entries, reversals, period close) require an API key for user attribution:
```bash
# Via flag
clawcounting journal-entries create --file entry.json --api-key tsk_... --json

# Via environment variable
export CLAWCOUNTING_API_KEY=tsk_...
clawcounting journal-entries create --file entry.json --json
```

Admin commands (user/currency/account/period creation, reports, settings) work without an API key.

**HTTP API** (requires server):
```
GET/POST/PATCH /api/v1/...
Authorization: Bearer <API_KEY or JWT>
```
Start with `clawcounting serve`. JWT secret is auto-generated on first run. See [references/setup-guide.md](references/setup-guide.md) for full server setup.

Both interfaces share identical validation and business logic.

## Amount Handling

All monetary amounts are **i128 integers in the smallest currency unit**:
- USD with `asset_scale=2`: `"1050"` = $10.50
- ETH with `asset_scale=18`: `"1000000000000000000"` = 1.0 ETH
- BTC with `asset_scale=8`: `"100000000"` = 1.0 BTC

API responses include both raw amounts and `display_*` fields formatted with the currency's decimal places.

## Key Rules

1. **Double-entry**: Every journal entry MUST balance (total debits = total credits). Minimum 2 lines. All lines same currency.
2. **Immutability**: Journal entries cannot be edited or deleted. Corrections are made via reversing entries only.
3. **Period enforcement**: Every entry must fall within an open financial period. Closed periods cannot be reopened.
4. **Control accounts**: Accounts with `has_subledger=true` cannot be posted to directly — post to their sub-accounts instead.
5. **Currency identifiers**: All currencies use CAIP-19 format (fiat: `swift:0/iso4217:USD`, crypto: `eip155:1/slip44:60`).

## Setup Workflow (first-time)

Follow these steps in order. See [references/setup-guide.md](references/setup-guide.md) for detailed examples.

1. **Start the server** or use CLI directly (both work with the same database)
2. **Create a user account** — via web UI setup page (first visit) or `clawcounting users create` CLI
3. **Create a service account** — `clawcounting users create-service-account` to get an API key for CLI write operations and agent access
4. **Add currencies** — use `currencies create-fiat` for common fiat, manual create for crypto
5. **Create a financial period** — define the date range for your fiscal period
6. **Build chart of accounts** — create asset, liability, equity, revenue, and expense accounts
7. **Configure settings** — set the retained earnings account (required for period close)

## Day-to-Day Operations

See [references/operations-guide.md](references/operations-guide.md) for detailed procedures.

- **Post journal entries** — balanced debit/credit entries with date, description, and line items
- **Query balances** — per-account, per-period, or cumulative
- **Generate reports** — trial balance, balance sheet, income statement, general ledger
- **Reverse entries** — create a correcting entry that swaps all debits and credits
- **Close periods** — preview the closing entry first, then commit (zeroes revenue/expense to retained earnings)

## API Reference

See [references/api-reference.md](references/api-reference.md) for all endpoints, request/response shapes, and query parameters.

## Accounting Rules & Constraints

See [references/accounting-rules.md](references/accounting-rules.md) for domain rules, subledger mechanics, currency handling, and error recovery.

## Error Handling

Errors follow RFC 7807 and always include a `suggestion` field with recovery guidance:
```json
{
  "code": "PERIOD_CLOSED",
  "message": "Period FY2025 is closed",
  "field": null,
  "suggestion": "Post to period FY2026 which is currently open"
}
```

Always read the `suggestion` before retrying. Common error codes:
- `VALIDATION_ERROR` — field-level validation failure
- `UNBALANCED_ENTRY` — debits != credits
- `PERIOD_CLOSED` — target period is closed
- `NOT_FOUND` — resource does not exist
- `UNAUTHORIZED` / `FORBIDDEN` — auth issues

## Response Envelope

Single resource:
```json
{ "data": { ... } }
```

List (cursor-paginated, Stripe-style):
```json
{ "data": [...], "has_more": true, "next_cursor": "..." }
```

Pagination: `?limit=50&cursor=<next_cursor>`. Default limit 50, max 200.
