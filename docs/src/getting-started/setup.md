# Setup Guide

Complete these steps in order after [installation](installation.md).

## 1. Initialize the Database

```bash
# Initialize in the default location (./clawcounting.db)
clawcounting init

# Or specify a path
clawcounting init --db /path/to/clawcounting.db
```

Running `init` on an existing database is safe -- it will apply any pending migrations.

## 2. Choose Your Interface

**CLI only** (no server needed):
```bash
clawcounting currencies list --json
```

**Server mode** (REST API + web UI):
```bash
clawcounting serve
# => Listening on 0.0.0.0:3000
```

Both interfaces share the same database and can be used simultaneously.

## 3. Create a Service Account

Service accounts authenticate with API keys. This is the recommended auth method for AI agents and CLI write operations.

```bash
clawcounting users create-service-account --name "AI Agent" --json
```

```json
{
  "data": {
    "user": {
      "id": "019...",
      "name": "AI Agent",
      "permissions": {},
      "is_active": true
    },
    "api_key": "tsk_a1b2c3d4..."
  }
}
```

> **Save the `api_key` immediately -- it is shown only once.**

Set it as an environment variable for convenience:
```bash
export CLAWCOUNTING_API_KEY=tsk_a1b2c3d4...
```

## 4. Add Currencies

Every account requires a currency. Add currencies before creating accounts.

### Web UI

Click **Add Currency** on the Currencies page. The dialog lets you:
- **Fiat** -- pick from a searchable list of ISO 4217 currencies (with country flags)
- **Crypto > Popular** -- pick from built-in native chains (BTC, ETH, SOL, etc.) and ~400 ERC-20 tokens (with logos)
- **Crypto > Import List** -- import tokens from a [Uniswap Token List](https://tokenlists.org/) JSON (URL or file upload) with a preview and bulk-add
- **Crypto > Custom** -- manually enter any currency

### CLI

```bash
# Fiat (auto-fills name, symbol, asset_scale, CAIP-19)
clawcounting currencies create-fiat USD --json

# Crypto (manual)
clawcounting currencies create \
  --code ETH \
  --name "Ether" \
  --symbol "Ξ" \
  --asset-scale 18 \
  --type crypto \
  --caip19 "eip155:1/slip44:60" \
  --json
```

See [Currencies & Amounts](../concepts/currencies.md) for details on CAIP-19 identifiers and asset scales.

## 5. Create a Financial Period

Every journal entry must fall within an open financial period.

```bash
clawcounting periods create \
  --name "FY2026" \
  --start 2026-01-01 \
  --end 2026-12-31 \
  --json
```

Periods must not overlap. They can be any duration (month, quarter, year). See [Financial Periods](../concepts/periods.md) for more.

## 6. Build Chart of Accounts

Create accounts for the types of transactions you'll record:

```bash
# Get your currency ID
CURRENCY_ID=$(clawcounting currencies list --json | jq -r '.data[0].id')

# Asset accounts
clawcounting accounts create --name "Cash" --currency $CURRENCY_ID \
  --type asset --normal-balance debit --number 1000 --json
clawcounting accounts create --name "Accounts Receivable" --currency $CURRENCY_ID \
  --type asset --normal-balance debit --number 1200 --json

# Liability accounts
clawcounting accounts create --name "Accounts Payable" --currency $CURRENCY_ID \
  --type liability --normal-balance credit --number 2000 --json

# Equity accounts
clawcounting accounts create --name "Owner's Equity" --currency $CURRENCY_ID \
  --type equity --normal-balance credit --number 3000 --json
clawcounting accounts create --name "Retained Earnings" --currency $CURRENCY_ID \
  --type equity --normal-balance credit --number 3100 --json

# Revenue accounts
clawcounting accounts create --name "Service Revenue" --currency $CURRENCY_ID \
  --type revenue --normal-balance credit --number 4000 --json

# Expense accounts
clawcounting accounts create --name "Rent Expense" --currency $CURRENCY_ID \
  --type expense --normal-balance debit --number 5000 --json
clawcounting accounts create --name "Wages Expense" --currency $CURRENCY_ID \
  --type expense --normal-balance debit --number 5100 --json
```

## 7. Configure Retained Earnings

Period close requires knowing which equity account to transfer net income into:

```bash
RE_ID=$(clawcounting accounts list --type equity --json | \
  jq -r '.data[] | select(.name == "Retained Earnings") | .id')

clawcounting settings set retained-earnings-account $RE_ID
```

## Setup Checklist

- [ ] Database initialized (`clawcounting init`)
- [ ] At least one currency created
- [ ] At least one open financial period
- [ ] Chart of accounts with asset, liability, equity, revenue, and expense accounts
- [ ] Retained earnings account configured in settings
- [ ] Service account created with API key (for CLI write ops and API access)
- [ ] `CLAWCOUNTING_API_KEY` set in environment
