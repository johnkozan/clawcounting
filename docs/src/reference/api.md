# API Reference

Complete REST API reference. Base URL: `http://localhost:3000`.

## Authentication

All `/api/v1/*` endpoints require authentication:

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

Only `name` and `symbol` are updatable.

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

### List Accounts

```
GET /api/v1/accounts?account_type=asset&currency_id=...&is_active=true&parent_id=...&limit=50&cursor=...
```

### Get Account

```
GET /api/v1/accounts/{id}
```

### Update Account

```
PATCH /api/v1/accounts/{id}
```

Only `name`, `is_active`, and `xbrl_tag` are updatable.

### Get Sub-Accounts

```
GET /api/v1/accounts/{id}/sub-accounts
```

### Get Account Balance

```
GET /api/v1/accounts/{id}/balance?period_id=<optional>
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

`entry_date` is optional -- defaults to original entry date. The reversal date must fall within an open period. A reversal cannot be reversed.

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

With `preview=true`: returns the closing entry without committing. Without `preview`: closes the period permanently.

---

## Reports

### Trial Balance

```
GET /api/v1/reports/trial-balance?period_id=<optional>&currency_id=<optional>
```

Returns account rows with debit/credit totals and `is_balanced` flag.

### Balance Sheet

```
GET /api/v1/reports/balance-sheet?period_id=<optional>&as_of_date=<optional>
```

Returns sections: `assets`, `liabilities`, `equity`. Includes `is_balanced` (Assets = Liabilities + Equity).

### Income Statement

```
GET /api/v1/reports/income-statement?period_id=<optional>
```

Returns: `revenue`, `expenses`, `total_revenue`, `total_expenses`, `net_income`.

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
