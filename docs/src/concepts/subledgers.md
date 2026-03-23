# Subledgers

Subledgers provide detail within a single line item on the trial balance -- for example, tracking receivables per customer or payables per vendor.

## How It Works

- A **control account** has `has_subledger=true` and acts as a summary.
- **Sub-accounts** have `parent_id` pointing to the control account and an `entity_id` identifying the counterparty.
- The control account balance is **computed** as the sum of all sub-account balances.
- You post to sub-accounts, never directly to the control account.

## Example: Accounts Receivable

```
Accounts Receivable (control, has_subledger=true)
  ├── AR - Acme Corp    (entity_id: acme)     balance: $3,000
  ├── AR - Beta Inc     (entity_id: beta)     balance: $1,500
  └── AR - Gamma LLC    (entity_id: gamma)    balance: $500

Control account balance (computed): $5,000
```

A journal entry for a sale to Acme Corp:
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

## Setup

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

# Create sub-accounts per entity
clawcounting accounts create \
  --name "AR - Acme Corp" \
  --number 1200-001 \
  --parent $AR_ID \
  --entity "acme-corp" \
  --json
```

Sub-accounts inherit `currency_id`, `account_type`, and `normal_balance` from the parent -- you don't need to specify them.

## Constraints

- Control accounts (`has_subledger=true`) cannot be posted to directly
- Sub-accounts require both `parent_id` and `entity_id`
- The parent must have `has_subledger=true`
- A control account cannot itself be a sub-account
- Sub-accounts inherit currency, type, and normal balance from the parent

## Querying

```bash
# Control account balance = sum of all sub-accounts
curl http://localhost:3000/api/v1/accounts/<control-id>/balance

# List sub-accounts
curl http://localhost:3000/api/v1/accounts/<control-id>/sub-accounts

# Individual sub-account balance
curl http://localhost:3000/api/v1/accounts/<sub-account-id>/balance
```

## Common Patterns

| Control Account | Entity ID | Purpose |
|----------------|-----------|---------|
| Accounts Receivable | Customer ID | Track amounts owed by each customer |
| Accounts Payable | Vendor ID | Track amounts owed to each vendor |
| Inventory | Product SKU | Track inventory by item |
| Fixed Assets | Asset tag/serial | Track individual fixed assets |
