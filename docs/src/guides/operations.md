# Operations Guide

Day-to-day accounting operations with ClawCounting.

## Posting Journal Entries

Journal entries are the core of double-entry bookkeeping. Every entry must balance: total debits = total credits.

### From a JSON File (CLI)

Create `entry.json`:

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

> Requires `CLAWCOUNTING_API_KEY` env var or `--api-key` flag for user attribution.

### Via API

```bash
curl -X POST http://localhost:3000/api/v1/journal-entries \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "entry_date": "2026-03-15",
    "description": "Monthly office rent",
    "lines": [
      {"account_id": "<rent-expense-id>", "debit_amount": "150000"},
      {"account_id": "<cash-id>", "credit_amount": "150000"}
    ]
  }'
```

### Multi-Line Entries

Entries can have more than 2 lines. Example -- payroll with tax withholding:

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

## Querying Balances

```bash
# All-time balance for an account
clawcounting accounts get <account-id> --json

# Period-specific balance (API)
curl "http://localhost:3000/api/v1/accounts/<id>/balance?period_id=<period-id>" \
  -H "Authorization: Bearer $API_KEY"
```

## Reversing Entries

Reversals are the only way to correct a posted journal entry. A reversal creates a new entry with all debits and credits swapped.

```bash
# Reverse with original date
clawcounting journal-entries reverse <entry-id> --json

# Reverse with a different date
clawcounting journal-entries reverse <entry-id> --date 2026-04-01 --json
```

After reversal, the net effect on all accounts is zero. Then post a new corrected entry.

## Generating Reports

### Trial Balance

Verifies that total debits equal total credits across all accounts.

```bash
clawcounting reports trial-balance --json
clawcounting reports trial-balance --period <period-id> --json
```

### Balance Sheet

Shows assets, liabilities, and equity. Verifies Assets = Liabilities + Equity.

```bash
clawcounting reports balance-sheet --json
clawcounting reports balance-sheet --as-of 2026-06-30 --json
```

### Income Statement

Shows revenue, expenses, and net income for a period.

```bash
clawcounting reports income-statement --period <period-id> --json
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

## Closing a Financial Period

See [Financial Periods](../concepts/periods.md#period-close) for the full process. The short version:

```bash
# Always preview first
clawcounting periods close <period-id> --preview --api-key $API_KEY --json

# Then close permanently
clawcounting periods close <period-id> --api-key $API_KEY --json
```

## Common Patterns

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
