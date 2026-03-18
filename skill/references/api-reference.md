# API Reference

Complete REST API reference for ClawCounting Accounting. Base URL: `http://localhost:3000`.

## Table of Contents

1. [Authentication](#authentication)
2. [Currencies](#currencies)
3. [Accounts](#accounts)
4. [Journal Entries](#journal-entries)
5. [Financial Periods](#financial-periods)
6. [Reports](#reports)
7. [Settings](#settings)
8. [Users](#users)

---

## Authentication

All `/api/v1/*` endpoints require authentication. Two methods:

| Method | Header | Use case |
|--------|--------|----------|
| API Key | `Authorization: Bearer tsk_...` | Service accounts (agents) |
| JWT | `Authorization: Bearer eyJ...` | Human users (web UI) |

### Login (JWT)

```
POST /auth/login
```

```json
{
  "email": "admin@example.com",
  "password": "secret"
}
```

Response:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "expires_in": 900
}
```

### Refresh Token

```
POST /auth/refresh
```

```json
{
  "refresh_token": "eyJ..."
}
```

### Current User

```
GET /auth/me
```

---

## Currencies

### Create Currency

```
POST /api/v1/currencies
```

```json
{
  "code": "USD",
  "name": "United States Dollar",
  "symbol": "$",
  "asset_scale": 2,
  "asset_type": "fiat",
  "caip19_id": "swift:0/iso4217:USD"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `code` | string | yes | Unique, e.g. "USD", "ETH" |
| `name` | string | yes | Display name |
| `symbol` | string | yes | e.g. "$", "Ξ" |
| `asset_scale` | integer | yes | Decimal places (2 for cents, 18 for wei) |
| `asset_type` | string | yes | `"fiat"` or `"crypto"` |
| `caip19_id` | string | yes | Unique CAIP-19 identifier |

### List Currencies

```
GET /api/v1/currencies?limit=50&cursor=<cursor>
```

### Get Currency

```
GET /api/v1/currencies/{id}
```

### Update Currency

```
PATCH /api/v1/currencies/{id}
```

```json
{
  "name": "US Dollar",
  "symbol": "$"
}
```

Only `name` and `symbol` are updatable. `code` and `caip19_id` are immutable.

---

## Accounts

### Create Account

```
POST /api/v1/accounts
```

```json
{
  "account_number": "1000",
  "name": "Cash",
  "currency_id": "019...",
  "account_type": "asset",
  "normal_balance": "debit"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `account_number` | string | yes | Unique identifier (e.g. "1000") |
| `name` | string | yes | Display name |
| `currency_id` | string | yes | Must reference an existing currency |
| `account_type` | string | yes | `asset`, `liability`, `equity`, `revenue`, `expense` |
| `normal_balance` | string | yes | `debit` or `credit` |
| `has_subledger` | bool | no | Enable sub-accounts (default: false) |
| `parent_id` | string | no | Parent control account (for sub-accounts) |
| `entity_id` | string | no | Entity identifier (required with parent_id) |
| `xbrl_tag` | string | no | XBRL classification tag |

**Sub-account rules**: If `parent_id` is set, `entity_id` must also be set. The parent must have `has_subledger=true`. Sub-accounts inherit `currency_id`, `account_type`, and `normal_balance` from the parent.

### List Accounts

```
GET /api/v1/accounts?account_type=asset&currency_id=...&is_active=true&parent_id=...&limit=50&cursor=...
```

All query parameters are optional filters.

### Get Account

```
GET /api/v1/accounts/{id}
```

### Update Account

```
PATCH /api/v1/accounts/{id}
```

```json
{
  "name": "Petty Cash",
  "is_active": false,
  "xbrl_tag": "us-gaap:Cash"
}
```

Only `name`, `is_active`, and `xbrl_tag` are updatable.

### Get Sub-Accounts

```
GET /api/v1/accounts/{id}/sub-accounts
```

Returns all sub-accounts for a control account.

### Get Account Balance

```
GET /api/v1/accounts/{id}/balance?period_id=<optional>
```

Response:
```json
{
  "data": {
    "account_id": "019...",
    "period_id": "019...",
    "total_debits": "250000",
    "total_credits": "100000",
    "net_balance": "150000",
    "display_debits": "2500.00",
    "display_credits": "1000.00",
    "display_balance": "1500.00"
  }
}
```

Without `period_id`, returns cumulative balance across all periods. For control accounts, returns sum of all sub-account balances.

### Get Account Transactions

```
GET /api/v1/accounts/{id}/transactions?limit=50&cursor=...
```

---

## Journal Entries

### Create Journal Entry

```
POST /api/v1/journal-entries
```

```json
{
  "entry_date": "2026-03-15",
  "description": "Monthly rent payment",
  "reference": "INV-001",
  "metadata": {"vendor": "Acme"},
  "lines": [
    {
      "account_id": "019...",
      "debit_amount": "150000",
      "description": "Rent expense"
    },
    {
      "account_id": "019...",
      "credit_amount": "150000",
      "description": "Cash payment"
    }
  ]
}
```

**Entry fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `entry_date` | string | yes | ISO 8601 date, must be in an open period |
| `description` | string | yes | Entry description |
| `reference` | string | no | External reference (invoice number, etc.) |
| `metadata` | object | no | Arbitrary JSON metadata |
| `lines` | array | yes | Minimum 2 lines |

**Line fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `account_id` | string | yes | Must be active, same currency as other lines |
| `debit_amount` | string | conditional | i128 in smallest unit. Exactly one of debit/credit per line. |
| `credit_amount` | string | conditional | i128 in smallest unit. Exactly one of debit/credit per line. |
| `description` | string | no | Line-level description |

### List Journal Entries

```
GET /api/v1/journal-entries?period_id=...&start_date=...&end_date=...&account_id=...&limit=50&cursor=...
```

### Get Journal Entry

```
GET /api/v1/journal-entries/{id}
```

Returns entry with all lines, including `display_debit` and `display_credit` formatted amounts.

### Reverse Journal Entry

```
POST /api/v1/journal-entries/{id}/reverse
```

```json
{
  "entry_date": "2026-04-01"
}
```

`entry_date` is optional — defaults to original entry date if omitted. The reversal date must fall within an open period. A reversal cannot be reversed.

---

## Financial Periods

### Create Period

```
POST /api/v1/periods
```

```json
{
  "name": "FY2026",
  "start_date": "2026-01-01",
  "end_date": "2026-12-31"
}
```

Periods must not overlap with existing periods.

### List Periods

```
GET /api/v1/periods?limit=50&cursor=...
```

### Get Period

```
GET /api/v1/periods/{id}
```

### Close Period

```
POST /api/v1/periods/{id}/close?preview=true
```

With `preview=true`: returns the closing entry without committing. Without `preview` (or `preview=false`): closes the period permanently.

Response:
```json
{
  "data": {
    "period": { "id": "019...", "name": "FY2026", "closed_at": "2026-04-01T..." },
    "closing_entry": { "id": "019...", "lines": [...] },
    "preview": false
  }
}
```

---

## Reports

### Trial Balance

```
GET /api/v1/reports/trial-balance?period_id=<optional>&currency_id=<optional>
```

Response:
```json
{
  "data": {
    "rows": [
      {
        "account_id": "019...",
        "account_number": "1000",
        "account_name": "Cash",
        "account_type": "asset",
        "debit_total": "500000",
        "credit_total": "200000",
        "display_debit_total": "5000.00",
        "display_credit_total": "2000.00"
      }
    ],
    "grand_total_debits": "750000",
    "grand_total_credits": "750000",
    "display_grand_total_debits": "7500.00",
    "display_grand_total_credits": "7500.00",
    "is_balanced": true
  }
}
```

### Balance Sheet

```
GET /api/v1/reports/balance-sheet?period_id=<optional>&as_of_date=<optional>
```

Returns sections: `assets`, `liabilities`, `equity`. Includes `is_balanced` (Assets = Liabilities + Equity).

### Income Statement

```
GET /api/v1/reports/income-statement?period_id=<optional>
```

Returns: `revenue`, `expenses`, `total_revenue`, `total_expenses`, `net_income` with display amounts.

### General Ledger

```
GET /api/v1/reports/general-ledger?account_id=<required>&period_id=<optional>&start_date=<optional>&end_date=<optional>&sort=desc&limit=50&cursor=...
```

Returns: `starting_balance`, paginated `lines` with `running_balance`, and `ending_balance`.

---

## Settings

### Get Settings

```
GET /api/v1/settings
```

### Update Settings

```
PATCH /api/v1/settings
```

```json
{
  "retained_earnings_account_id": "019..."
}
```

The retained earnings account must be an equity-type account.

---

## Users

### Create Human User

```
POST /api/v1/users
```

```json
{
  "name": "Admin",
  "email": "admin@example.com",
  "password": "secure-password",
  "permissions": {}
}
```

### Create Service Account

```
POST /api/v1/users/service-accounts
```

```json
{
  "name": "AI Agent",
  "permissions": {}
}
```

Response includes `api_key` (shown only once).

### List Users

```
GET /api/v1/users?limit=50&cursor=...
```

### Get User

```
GET /api/v1/users/{id}
```

### Update User

```
PATCH /api/v1/users/{id}
```

```json
{
  "name": "New Name",
  "permissions": {"journals:create": true},
  "is_active": false
}
```

---

## CLI Equivalents

Every API endpoint has a CLI equivalent. Add `--json` for machine-readable output:

| API Endpoint | CLI Command |
|-------------|-------------|
| `POST /api/v1/currencies` | `clawcounting currencies create ...` |
| `GET /api/v1/currencies` | `clawcounting currencies list` |
| N/A (fiat shortcut) | `clawcounting currencies create-fiat USD` |
| `POST /api/v1/accounts` | `clawcounting accounts create ...` |
| `GET /api/v1/accounts` | `clawcounting accounts list` |
| `POST /api/v1/journal-entries` | `clawcounting journal-entries create --file entry.json` |
| `GET /api/v1/journal-entries` | `clawcounting journal-entries list` |
| `POST /api/v1/journal-entries/{id}/reverse` | `clawcounting journal-entries reverse <id>` |
| `POST /api/v1/periods` | `clawcounting periods create ...` |
| `POST /api/v1/periods/{id}/close` | `clawcounting periods close <id>` |
| `GET /api/v1/reports/trial-balance` | `clawcounting reports trial-balance` |
| `GET /api/v1/reports/balance-sheet` | `clawcounting reports balance-sheet` |
| `GET /api/v1/reports/income-statement` | `clawcounting reports income-statement` |
| `GET /api/v1/reports/general-ledger` | `clawcounting reports general-ledger` |
| `PATCH /api/v1/settings` | `clawcounting settings set ...` |
