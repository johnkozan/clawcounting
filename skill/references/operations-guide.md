# Operations Guide

Day-to-day accounting operations with ClawCounting.

## Table of Contents

1. [Posting Journal Entries](#1-posting-journal-entries)
2. [Querying Balances](#2-querying-balances)
3. [Reversing Entries](#3-reversing-entries)
4. [Generating Reports](#4-generating-reports)
5. [Closing a Financial Period](#5-closing-a-financial-period)
6. [Working with Subledgers](#6-working-with-subledgers)
7. [Common Patterns](#7-common-patterns)

---

## 1. Posting Journal Entries

Journal entries are the core of double-entry bookkeeping. Every entry must balance: total debits = total credits.

### Rules
- Minimum 2 lines (at least one debit and one credit)
- All line amounts are in the smallest currency unit (cents for USD, wei for ETH)
- All lines must reference accounts in the same currency
- Entry date must fall within an open period
- Cannot post to control accounts (has_subledger=true) — use sub-accounts
- Entries are immutable once posted — corrections via reversal only

### CLI (from JSON file)

Create a JSON file (`entry.json`):
```json
{
  "entry_date": "2026-03-15",
  "description": "Monthly office rent",
  "reference": "INV-2026-0042",
  "metadata": {"vendor": "Acme Properties"},
  "lines": [
    {
      "account_id": "<rent-expense-account-id>",
      "debit_amount": "150000",
      "description": "March 2026 rent"
    },
    {
      "account_id": "<cash-account-id>",
      "credit_amount": "150000",
      "description": "Payment for March rent"
    }
  ]
}
```

Post it:
```bash
clawcounting journal-entries create --file entry.json --json
```

### API

```bash
curl -X POST http://localhost:3000/api/v1/journal-entries \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "entry_date": "2026-03-15",
    "description": "Monthly office rent",
    "reference": "INV-2026-0042",
    "lines": [
      {
        "account_id": "<rent-expense-id>",
        "debit_amount": "150000"
      },
      {
        "account_id": "<cash-id>",
        "credit_amount": "150000"
      }
    ]
  }'
```

### Response

```json
{
  "data": {
    "id": "019...",
    "period_id": "019...",
    "entry_date": "2026-03-15",
    "posted_at": "2026-03-15T14:30:00Z",
    "description": "Monthly office rent",
    "reference": "INV-2026-0042",
    "is_reversal": false,
    "reverses_id": null,
    "metadata": {"vendor": "Acme Properties"},
    "lines": [
      {
        "id": "019...",
        "account_id": "019...",
        "debit_amount": "150000",
        "credit_amount": "0",
        "display_debit": "1500.00",
        "display_credit": "0.00",
        "description": "March 2026 rent"
      },
      {
        "id": "019...",
        "account_id": "019...",
        "debit_amount": "0",
        "credit_amount": "150000",
        "display_debit": "0.00",
        "display_credit": "1500.00",
        "description": "Payment for March rent"
      }
    ]
  }
}
```

### Multi-line entries

Entries can have more than 2 lines. Example — payroll with tax withholding:

```json
{
  "entry_date": "2026-03-31",
  "description": "March payroll",
  "lines": [
    {"account_id": "<wages-expense-id>", "debit_amount": "500000"},
    {"account_id": "<tax-payable-id>", "credit_amount": "75000"},
    {"account_id": "<cash-id>", "credit_amount": "425000"}
  ]
}
```

Debits (500000) = Credits (75000 + 425000). Entry balances.

---

## 2. Querying Balances

### Single account balance

```bash
# All-time balance
clawcounting accounts get <account-id> --json

# Period-specific balance
curl http://localhost:3000/api/v1/accounts/<id>/balance?period_id=<period-id> \
  -H "Authorization: Bearer $API_KEY"
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

### Account transactions

```bash
curl "http://localhost:3000/api/v1/accounts/<id>/transactions?limit=20" \
  -H "Authorization: Bearer $API_KEY"
```

---

## 3. Reversing Entries

Reversals are the only way to correct a posted journal entry. A reversal creates a new entry with all debits and credits swapped.

### CLI

```bash
# Reverse with original date
clawcounting journal-entries reverse <entry-id> --json

# Reverse with a different date (e.g., correction in current period)
clawcounting journal-entries reverse <entry-id> --date 2026-04-01 --json
```

### API

```bash
curl -X POST http://localhost:3000/api/v1/journal-entries/<id>/reverse \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"entry_date": "2026-04-01"}'
```

### Rules
- The reversal date must fall within an open period
- A reversal cannot itself be reversed
- The original entry is not modified — it remains in the ledger
- After reversal, the net effect on all accounts is zero

### Correction workflow

To correct an error:
1. **Reverse** the original entry
2. **Post** a new entry with the correct amounts

---

## 4. Generating Reports

### Trial Balance

Verifies that total debits equal total credits across all accounts.

```bash
# All-time
clawcounting reports trial-balance --json

# For a specific period
clawcounting reports trial-balance --period <period-id> --json
```

API:
```bash
curl "http://localhost:3000/api/v1/reports/trial-balance?period_id=<id>" \
  -H "Authorization: Bearer $API_KEY"
```

Response includes `is_balanced: true` when debits equal credits.

### Balance Sheet

Shows assets, liabilities, and equity. Verifies the accounting equation: Assets = Liabilities + Equity.

```bash
# Current (all periods)
clawcounting reports balance-sheet --json

# As of a specific date
clawcounting reports balance-sheet --as-of 2026-06-30 --json

# For a specific period
clawcounting reports balance-sheet --period <period-id> --json
```

API:
```bash
curl "http://localhost:3000/api/v1/reports/balance-sheet?as_of_date=2026-06-30" \
  -H "Authorization: Bearer $API_KEY"
```

Response includes `is_balanced: true` when the accounting equation holds.

### Income Statement

Shows revenue, expenses, and net income for a period.

```bash
clawcounting reports income-statement --period <period-id> --json
```

API:
```bash
curl "http://localhost:3000/api/v1/reports/income-statement?period_id=<id>" \
  -H "Authorization: Bearer $API_KEY"
```

### General Ledger

Detailed transaction history for a single account with running balance.

```bash
clawcounting reports general-ledger \
  --account <account-id> \
  --start 2026-01-01 \
  --end 2026-03-31 \
  --json
```

API:
```bash
curl "http://localhost:3000/api/v1/reports/general-ledger?account_id=<id>&start_date=2026-01-01&end_date=2026-03-31" \
  -H "Authorization: Bearer $API_KEY"
```

Response includes `starting_balance`, per-line `running_balance`, and `ending_balance`.

---

## 5. Closing a Financial Period

Period close zeroes all revenue and expense accounts, transferring net income to retained earnings.

### Prerequisites
- `retained_earnings_account_id` must be set in settings (see [setup guide](setup-guide.md#6-configure-settings))
- The retained earnings account must be an equity-type account
- Period must be open (not already closed)

### Step 1: Preview

Always preview first. This shows what the closing entry will look like without committing:

```bash
clawcounting periods close <period-id> --preview --json
```

API:
```bash
curl -X POST "http://localhost:3000/api/v1/periods/<id>/close?preview=true" \
  -H "Authorization: Bearer $API_KEY"
```

Review the closing entry lines — each revenue/expense account will have a line that zeroes its balance, and one line transfers the net amount to retained earnings.

### Step 2: Close

```bash
clawcounting periods close <period-id> --json
```

API:
```bash
curl -X POST "http://localhost:3000/api/v1/periods/<id>/close" \
  -H "Authorization: Bearer $API_KEY"
```

### What happens on close

1. For each **revenue** account with a balance: a debit line zeroes the credit balance
2. For each **expense** account with a balance: a credit line zeroes the debit balance
3. One line posts the net income (or loss) to the retained earnings account
4. The period is marked closed with a timestamp
5. The auto-generated closing journal entry is linked to the period

### After closing
- No new entries can be posted to the closed period
- The period cannot be reopened
- Post adjustments to a different open period
- Asset, liability, and equity balances carry forward naturally

---

## 6. Working with Subledgers

Subledgers let you track detail within a single account — e.g., receivables per customer or payables per vendor.

### Setup

```bash
# Create control account with subledger enabled
clawcounting accounts create \
  --name "Accounts Receivable" \
  --currency $CURRENCY_ID \
  --type asset \
  --normal-balance debit \
  --number 1200 \
  --subledger \
  --json

# Create sub-accounts per entity
clawcounting accounts create \
  --name "AR - Acme Corp" \
  --number 1200-001 \
  --parent $AR_ID \
  --entity "acme-corp" \
  --json
```

Sub-accounts inherit `currency_id`, `account_type`, and `normal_balance` from the parent.

### Posting

Always post to **sub-accounts**, never to the control account:

```json
{
  "entry_date": "2026-03-15",
  "description": "Invoice #1001 to Acme Corp",
  "lines": [
    {"account_id": "<ar-acme-sub-account-id>", "debit_amount": "250000"},
    {"account_id": "<revenue-id>", "credit_amount": "250000"}
  ]
}
```

### Querying

```bash
# Control account balance = sum of all sub-accounts
curl http://localhost:3000/api/v1/accounts/<control-id>/balance \
  -H "Authorization: Bearer $API_KEY"

# List sub-accounts
curl http://localhost:3000/api/v1/accounts/<control-id>/sub-accounts \
  -H "Authorization: Bearer $API_KEY"

# Individual sub-account balance
curl http://localhost:3000/api/v1/accounts/<sub-account-id>/balance \
  -H "Authorization: Bearer $API_KEY"
```

---

## 7. Common Patterns

### Record a sale

```json
{
  "entry_date": "2026-03-15",
  "description": "Sale to customer",
  "reference": "INV-001",
  "lines": [
    {"account_id": "<cash-or-ar-id>", "debit_amount": "100000"},
    {"account_id": "<revenue-id>", "credit_amount": "100000"}
  ]
}
```

### Record an expense

```json
{
  "entry_date": "2026-03-15",
  "description": "Office supplies",
  "lines": [
    {"account_id": "<supplies-expense-id>", "debit_amount": "5000"},
    {"account_id": "<cash-id>", "credit_amount": "5000"}
  ]
}
```

### Transfer between accounts

```json
{
  "entry_date": "2026-03-15",
  "description": "Transfer from checking to savings",
  "lines": [
    {"account_id": "<savings-id>", "debit_amount": "1000000"},
    {"account_id": "<checking-id>", "credit_amount": "1000000"}
  ]
}
```

### Record a loan payment (principal + interest)

```json
{
  "entry_date": "2026-03-15",
  "description": "Monthly loan payment",
  "lines": [
    {"account_id": "<loan-payable-id>", "debit_amount": "80000"},
    {"account_id": "<interest-expense-id>", "debit_amount": "20000"},
    {"account_id": "<cash-id>", "credit_amount": "100000"}
  ]
}
```

### Correct a mistake

```bash
# 1. Reverse the wrong entry
clawcounting journal-entries reverse <wrong-entry-id> --json

# 2. Post the corrected entry
clawcounting journal-entries create --file corrected-entry.json --json
```
