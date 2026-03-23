# Setup Guide

Step-by-step guide for setting up ClawCounting from scratch. Complete these steps in order.

## Table of Contents

1. [Initialize the Database](#1-initialize-the-database)
2. [Create a Service Account](#2-create-a-service-account)
3. [Add Currencies](#3-add-currencies)
4. [Create Financial Periods](#4-create-financial-periods)
5. [Build Chart of Accounts](#5-build-chart-of-accounts)
6. [Configure Settings](#6-configure-settings)

---

## 1. Initialize the Database

Before using ClawCounting, you must create and initialize the database with `clawcounting init`.

ClawCounting has two interfaces: a CLI that connects directly to the SQLite database, and an HTTP server that provides the REST API and web UI. Both share the same database.

### Environment variables

| Variable | Default | Required | Description |
|----------|---------|----------|-------------|
| `CLAWCOUNTING_DB` | `./clawcounting.db` | No | SQLite database file path |
| `CLAWCOUNTING_PORT` | `3000` | No | HTTP server port |
| `CLAWCOUNTING_JWT_SECRET` | Auto-generated | No | JWT signing secret. If not set, a random secret is auto-generated on first run and stored in the DB settings table. Set this explicitly only if you need stable tokens across database resets. |
| `CLAWCOUNTING_API_KEY` | — | No | API key for CLI commands that create accounting records (journal entries, reversals, period close). Alternative to `--api-key` flag. |

You can set these as environment variables or place them in a `.env` file in the working directory. The `.env` file is loaded automatically.

### Create the database

```bash
# Initialize in the default location (./clawcounting.db)
clawcounting init

# Initialize at a specific path
clawcounting init --db /path/to/clawcounting.db

# Or use the environment variable
export CLAWCOUNTING_DB=/data/clawcounting.db
clawcounting init
```

Running `init` on an existing database is safe — it will apply any pending migrations.

### Option A: CLI only (no server needed)

The CLI connects directly to the SQLite database. No JWT secret is needed.

```bash
clawcounting init
clawcounting currencies list --json
```

### Option B: Server mode (REST API + web UI)

Initialize the database first, then start the server:

```bash
clawcounting init
clawcounting serve
# => Listening on 0.0.0.0:3000
```

### Running the server in the background (for AI agents)

When an AI agent needs to start the server as part of a workflow, use `nohup` to detach the process and poll the health endpoint for readiness:

```bash
# Start server in background (survives shell exit)
nohup clawcounting serve > /tmp/clawcounting.log 2>&1 &
echo $! > /tmp/clawcounting.pid

# Wait for the server to be ready
until curl -sf http://localhost:3000/health > /dev/null 2>&1; do sleep 0.5; done

# Server is now accepting requests
```

To stop the server later:
```bash
kill "$(cat /tmp/clawcounting.pid)"
```

The JWT secret is auto-generated during `init` and stored in the database. Optionally, create a `.env` file to customize settings:

```bash
cat > .env << 'EOF'
CLAWCOUNTING_DB=./clawcounting.db
CLAWCOUNTING_PORT=3000
EOF

clawcounting init
clawcounting serve
```

The server provides:
- REST API at `/api/v1/*`
- Web UI at `/`
- Swagger API docs at `/swagger-ui/`
- Health check at `/health`

### Using both interfaces

CLI and server can use the same database file simultaneously. The CLI is useful for quick operations and scripting, while the server is needed for the web UI and remote API access.

---

## 2. Create a Service Account

Service accounts authenticate with API keys. This is the recommended auth method for AI agents.

### CLI

```bash
clawcounting users create-service-account --name "AI Agent" --json
```

Response:
```json
{
  "data": {
    "user": {
      "id": "019...",
      "name": "AI Agent",
      "email": null,
      "permissions": {},
      "is_active": true,
      "created_at": "2026-01-15T10:00:00Z"
    },
    "api_key": "tsk_a1b2c3d4..."
  }
}
```

**Save the `api_key` immediately — it is shown only once.**

Use the API key for CLI write operations and HTTP API requests:
```bash
# CLI: set as env var (recommended)
export CLAWCOUNTING_API_KEY=tsk_a1b2c3d4...
clawcounting journal-entries create --file entry.json --json

# CLI: pass as flag
clawcounting journal-entries create --file entry.json --api-key tsk_a1b2c3d4... --json

# HTTP API
curl -H "Authorization: Bearer tsk_a1b2c3d4..." http://localhost:3000/api/v1/accounts
```

### API

```bash
curl -X POST http://localhost:3000/api/v1/users/service-accounts \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"name": "AI Agent", "permissions": {}}'
```

---

## 3. Add Currencies

Every account requires a currency. Add currencies before creating accounts.

### Web UI

Click **Add Currency** on the Currencies page. The dialog provides:

- **Fiat tab** — searchable picker of all ISO 4217 currencies with country flags. Click to add.
- **Crypto tab** with three sub-tabs:
  - **Popular** — searchable picker of native chains (BTC, ETH, SOL, etc.) and ~400 ERC-20 tokens from the Uniswap Default token list, with logos.
  - **Import List** — import tokens from any Uniswap Token List standard JSON (paste a URL or upload a file). Shows a preview table with checkboxes for bulk-adding.
  - **Custom** — manual form for any currency not covered by the built-in lists.

### Fiat currencies (CLI quick method)

Use `create-fiat` with an ISO 4217 code — ClawCounting auto-fills name, symbol, asset_scale, and CAIP-19:

```bash
clawcounting currencies create-fiat USD --json
```

Response:
```json
{
  "data": {
    "id": "019...",
    "code": "USD",
    "name": "United States Dollar",
    "symbol": "$",
    "asset_scale": 2,
    "asset_type": "fiat",
    "caip19_id": "swift:0/iso4217:USD"
  }
}
```

Common fiat codes: `USD`, `EUR`, `GBP`, `JPY`, `CAD`, `AUD`, `CHF`.

### Crypto currencies (CLI manual)

```bash
clawcounting currencies create \
  --code ETH \
  --name "Ether" \
  --symbol "Ξ" \
  --asset-scale 18 \
  --type crypto \
  --caip19 "eip155:1/slip44:60" \
  --json
```

### CAIP-19 identifier format

| Type | Pattern | Example |
|------|---------|---------|
| Fiat | `swift:0/iso4217:<CODE>` | `swift:0/iso4217:USD` |
| Native coin (ETH) | `eip155:<chain>/slip44:<coin>` | `eip155:1/slip44:60` |
| ERC-20 token | `eip155:<chain>/erc20:<address>` | `eip155:1/erc20:0xa0b8...` |
| Bitcoin | `bip122:<genesis>/slip44:0` | `bip122:000000000019d6689c085ae165831e93/slip44:0` |

### Asset scale reference

| Currency | asset_scale | 1.0 stored as |
|----------|------------|---------------|
| USD | 2 | `100` |
| JPY | 0 | `1` |
| BTC | 8 | `100000000` |
| ETH | 18 | `1000000000000000000` |

### API

```bash
# Fiat (no create-fiat shortcut — provide all fields)
curl -X POST http://localhost:3000/api/v1/currencies \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "USD",
    "name": "United States Dollar",
    "symbol": "$",
    "asset_scale": 2,
    "asset_type": "fiat",
    "caip19_id": "swift:0/iso4217:USD"
  }'
```

---

## 4. Create Financial Periods

Financial periods define the date ranges for accounting activity. Every journal entry must fall within an open period.

### Rules
- `start_date` must be before `end_date`
- Periods must not overlap
- Closed periods cannot be reopened
- Periods can be any duration (month, quarter, year)

### CLI

```bash
clawcounting periods create \
  --name "FY2026" \
  --start 2026-01-01 \
  --end 2026-12-31 \
  --json
```

Response:
```json
{
  "data": {
    "id": "019...",
    "name": "FY2026",
    "start_date": "2026-01-01",
    "end_date": "2026-12-31",
    "closed_at": null,
    "closed_by": null,
    "closing_entry_id": null
  }
}
```

### Quarterly example

```bash
clawcounting periods create --name "Q1 2026" --start 2026-01-01 --end 2026-03-31 --json
clawcounting periods create --name "Q2 2026" --start 2026-04-01 --end 2026-06-30 --json
clawcounting periods create --name "Q3 2026" --start 2026-07-01 --end 2026-09-30 --json
clawcounting periods create --name "Q4 2026" --start 2026-10-01 --end 2026-12-31 --json
```

### API

```bash
curl -X POST http://localhost:3000/api/v1/periods \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "FY2026", "start_date": "2026-01-01", "end_date": "2026-12-31"}'
```

---

## 5. Build Chart of Accounts

### Account types

| Type | Normal Balance | Description |
|------|---------------|-------------|
| `asset` | debit | What the entity owns (cash, receivables, equipment) |
| `liability` | credit | What the entity owes (payables, loans, deferred revenue) |
| `equity` | credit | Owner's stake (capital, retained earnings) |
| `revenue` | credit | Income earned (sales, service fees, interest) |
| `expense` | debit | Costs incurred (rent, wages, utilities) |

### Minimal chart of accounts

At minimum, you need accounts for the types of transactions you'll record. A basic setup:

```bash
# Get currency ID first
CURRENCY_ID=$(clawcounting currencies list --json | jq -r '.data[0].id')

# Asset accounts
clawcounting accounts create --name "Cash" --currency $CURRENCY_ID --type asset --normal-balance debit --number 1000 --json
clawcounting accounts create --name "Accounts Receivable" --currency $CURRENCY_ID --type asset --normal-balance debit --number 1200 --json

# Liability accounts
clawcounting accounts create --name "Accounts Payable" --currency $CURRENCY_ID --type liability --normal-balance credit --number 2000 --json

# Equity accounts
clawcounting accounts create --name "Owner's Equity" --currency $CURRENCY_ID --type equity --normal-balance credit --number 3000 --json
clawcounting accounts create --name "Retained Earnings" --currency $CURRENCY_ID --type equity --normal-balance credit --number 3100 --json

# Revenue accounts
clawcounting accounts create --name "Service Revenue" --currency $CURRENCY_ID --type revenue --normal-balance credit --number 4000 --json

# Expense accounts
clawcounting accounts create --name "Rent Expense" --currency $CURRENCY_ID --type expense --normal-balance debit --number 5000 --json
clawcounting accounts create --name "Wages Expense" --currency $CURRENCY_ID --type expense --normal-balance debit --number 5100 --json
```

### Subledger accounts (AR/AP by customer or vendor)

Enable subledgers for accounts that track many counterparties:

```bash
# Create control account with subledger flag
clawcounting accounts create \
  --name "Accounts Receivable" \
  --currency $CURRENCY_ID \
  --type asset \
  --normal-balance debit \
  --number 1200 \
  --subledger \
  --json

AR_ID=$(...)  # Save the control account ID

# Create sub-accounts per customer
clawcounting accounts create \
  --name "AR - Acme Corp" \
  --currency $CURRENCY_ID \
  --type asset \
  --normal-balance debit \
  --number 1200-001 \
  --parent $AR_ID \
  --entity "acme-corp" \
  --json
```

**Important**: Post journal entries to sub-accounts, not the control account. The control account balance automatically sums all children.

### API

```bash
curl -X POST http://localhost:3000/api/v1/accounts \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "account_number": "1000",
    "name": "Cash",
    "currency_id": "019...",
    "account_type": "asset",
    "normal_balance": "debit"
  }'
```

---

## 6. Configure Settings

### Set retained earnings account (required for period close)

Period close transfers net income from revenue/expense accounts into retained earnings. You must configure which equity account to use:

```bash
# Get the retained earnings account ID
RE_ID=$(clawcounting accounts list --type equity --json | jq -r '.data[] | select(.name == "Retained Earnings") | .id')

clawcounting settings set retained-earnings-account $RE_ID
```

### API

```bash
curl -X PATCH http://localhost:3000/api/v1/settings \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"retained_earnings_account_id": "019..."}'
```

---

## Setup Checklist

- [ ] ClawCounting binary installed and on PATH
- [ ] Database initialized (`clawcounting init`)
- [ ] At least one currency created
- [ ] At least one open financial period
- [ ] Chart of accounts with asset, liability, equity, revenue, and expense accounts
- [ ] Retained earnings account configured in settings
- [ ] Service account created with API key (required for CLI journal entries/reversals/period close, and for API access)
- [ ] `CLAWCOUNTING_API_KEY` set in environment (if using CLI for accounting operations)
