# Accounting Rules & Constraints

Domain rules that ClawCounting enforces. Understanding these prevents errors.

## Table of Contents

1. [Double-Entry Bookkeeping](#1-double-entry-bookkeeping)
2. [Account Types & Normal Balances](#2-account-types--normal-balances)
3. [Amount Handling](#3-amount-handling)
4. [Currency System](#4-currency-system)
5. [Period Rules](#5-period-rules)
6. [Immutability](#6-immutability)
7. [Subledger Mechanics](#7-subledger-mechanics)
8. [Error Codes & Recovery](#8-error-codes--recovery)

---

## 1. Double-Entry Bookkeeping

Every financial transaction is recorded as a journal entry with two or more lines. The fundamental rule:

> **Total debits must equal total credits** in every journal entry.

This is enforced at the database level. An unbalanced entry will be rejected with error code `UNBALANCED_ENTRY`.

### Debit vs Credit

| Account Type | Debit increases | Credit increases |
|-------------|----------------|-----------------|
| Asset | Balance goes up | Balance goes down |
| Expense | Balance goes up | Balance goes down |
| Liability | Balance goes down | Balance goes up |
| Equity | Balance goes down | Balance goes up |
| Revenue | Balance goes down | Balance goes up |

**Mnemonic**: Assets and Expenses are "debit-normal" — they increase with debits. Everything else is "credit-normal."

### Examples

**Receive cash for services rendered:**
- Debit Cash (asset goes up)
- Credit Revenue (revenue goes up)

**Pay rent:**
- Debit Rent Expense (expense goes up)
- Credit Cash (asset goes down)

**Take out a loan:**
- Debit Cash (asset goes up)
- Credit Loan Payable (liability goes up)

---

## 2. Account Types & Normal Balances

| Type | Normal Balance | Accounting Equation Side | Closed at Period End? |
|------|---------------|-------------------------|----------------------|
| `asset` | `debit` | Left (A) | No — permanent |
| `liability` | `credit` | Right (L) | No — permanent |
| `equity` | `credit` | Right (E) | No — permanent |
| `revenue` | `credit` | Income Statement | Yes — zeroed to retained earnings |
| `expense` | `debit` | Income Statement | Yes — zeroed to retained earnings |

**The accounting equation**: Assets = Liabilities + Equity

ClawCounting verifies this equation in the balance sheet report (`is_balanced` field).

### Permanent vs Temporary accounts
- **Permanent** (asset, liability, equity): Balances carry forward across periods
- **Temporary** (revenue, expense): Balances are zeroed when a period is closed, with net income transferred to retained earnings

---

## 3. Amount Handling

### Internal representation
All amounts are stored as **i128 integers** in the smallest currency unit:
- No floating point — zero rounding errors
- Maximum value: ±170,141,183,460,469,231,731,687,303,715,884,105,727
- Stored as 16-byte BLOBs in SQLite with MSB-flipped encoding for correct sort order

### Converting between display and stored amounts

The `asset_scale` on each currency defines decimal placement:

| Display | asset_scale | Stored i128 |
|---------|------------|-------------|
| `$10.50` | 2 | `1050` |
| `¥1000` | 0 | `1000` |
| `1.5 ETH` | 18 | `1500000000000000000` |
| `0.001 BTC` | 8 | `100000` |

**Formula**: `stored = display × 10^asset_scale`

### In API requests
Amounts are sent as **string representations of the i128 value**:
```json
{"debit_amount": "1050"}
```
Not `"10.50"` — the raw integer in smallest unit.

### In API responses
Both forms are provided:
```json
{
  "debit_amount": "1050",
  "display_debit": "10.50"
}
```

---

## 4. Currency System

### CAIP-19 identifiers

Every currency has a unique [CAIP-19](https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-19.md) identifier:

| Type | Pattern | Example |
|------|---------|---------|
| Fiat (ISO 4217) | `swift:0/iso4217:<CODE>` | `swift:0/iso4217:USD` |
| Native coin | `eip155:<chain>/slip44:<coin>` | `eip155:1/slip44:60` (ETH) |
| ERC-20 token | `eip155:<chain>/erc20:<address>` | `eip155:1/erc20:0xa0b8...` |
| Bitcoin | `bip122:<genesis>/slip44:0` | (BTC mainnet) |

### Currency constraints
- `code` is unique and immutable after creation
- `caip19_id` is unique and immutable after creation
- `asset_scale` is immutable (changing it would corrupt all existing balances)
- Only `name` and `symbol` can be updated

### Multi-currency rules
- All lines in a single journal entry must reference accounts with the **same currency**
- Cross-currency transactions require separate entries or a conversion pattern
- Balance and report queries can filter by `currency_id`
- The balance sheet mixes currencies by default — use `currency_id` filter for single-currency views

### Cross-currency pattern

To record a currency exchange (e.g., sell USD for EUR), use two separate entries or a clearing account:

1. Debit "Currency Exchange" clearing account in USD
2. Credit USD Cash
3. (Separate entry in EUR currency)
4. Debit EUR Cash
5. Credit "Currency Exchange" clearing account in EUR

---

## 5. Period Rules

### Constraints
- Start date must be before end date
- No two periods may overlap (checked via `start_date < new_end AND end_date > new_start`)
- Periods can be any duration (day, month, quarter, year)
- Gaps between periods are allowed

### Open vs Closed
- **Open**: Accepts new journal entries within its date range
- **Closed**: Permanently sealed. No new entries, no reopening.

### Entry date validation
When posting a journal entry, ClawCounting:
1. Looks up which period contains the `entry_date`
2. If no period covers that date → error
3. If the period is closed → error with `PERIOD_CLOSED` code
4. The entry is linked to that period

### Period close process
1. All revenue accounts are debited to zero their credit balances
2. All expense accounts are credited to zero their debit balances
3. The net amount (revenue - expenses) is posted to retained earnings
4. An automatic closing journal entry is created and linked to the period
5. The period is marked closed with timestamp and user ID

See [operations guide](operations-guide.md#5-closing-a-financial-period) for step-by-step instructions.

---

## 6. Immutability

### What cannot be changed
- **Journal entries**: No UPDATE, no DELETE. Once posted, the entry exists permanently.
- **Journal entry lines**: Same — append-only.
- **Closed periods**: Cannot reopen.
- **Currency code/caip19_id/asset_scale**: Immutable after creation.
- **Account type/normal_balance/currency_id**: Immutable after creation.

### How to correct mistakes
1. **Reverse** the incorrect entry (creates a new entry with swapped debits/credits)
2. **Post** a new entry with correct amounts

This preserves a complete audit trail — both the error and the correction are visible.

### What can be changed
- Currency `name` and `symbol`
- Account `name`, `is_active`, and `xbrl_tag`
- User `name`, `permissions`, and `is_active`
- Settings values

---

## 7. Subledger Mechanics

Subledgers provide detail within a single line item on the trial balance.

### How it works
- A **control account** has `has_subledger=true` and acts as a summary
- **Sub-accounts** have `parent_id` pointing to the control account and an `entity_id` identifying the counterparty
- Control account balance = sum of all sub-account balances (automatic)
- You post to sub-accounts, never directly to the control account

### Constraints
- Control accounts (`has_subledger=true`) cannot be posted to directly → `VALIDATION_ERROR`
- Sub-accounts require both `parent_id` and `entity_id`
- Parent must have `has_subledger=true`
- Sub-accounts inherit `currency_id`, `account_type`, and `normal_balance` from parent
- A control account cannot itself be a sub-account (`parent_id` must be null)

### Common subledger patterns
| Control Account | Entity ID | Purpose |
|----------------|-----------|---------|
| Accounts Receivable | Customer name/ID | Track amounts owed by each customer |
| Accounts Payable | Vendor name/ID | Track amounts owed to each vendor |
| Inventory | Product SKU | Track inventory by item |
| Fixed Assets | Asset tag/serial | Track individual fixed assets |

---

## 8. Error Codes & Recovery

| Code | HTTP Status | Meaning | Recovery |
|------|------------|---------|----------|
| `VALIDATION_ERROR` | 400 | Field validation failed | Read `field` and `suggestion` — fix the specific field |
| `UNBALANCED_ENTRY` | 400 | Total debits ≠ total credits | Check line amounts. Ensure they sum to equal values. |
| `PERIOD_CLOSED` | 409 | Period is closed | Post to a different open period |
| `NOT_FOUND` | 404 | Resource doesn't exist | Verify the ID. List resources to find the correct one. |
| `UNAUTHORIZED` | 401 | Missing or invalid auth | Check Authorization header. Re-login or use valid API key. |
| `FORBIDDEN` | 403 | Insufficient permissions | Check user permissions. Request access from admin. |
| `DATABASE_ERROR` | 500 | Database failure | Retry. If persistent, check disk space and database integrity. |
| `INTERNAL_ERROR` | 500 | Unexpected error | Retry. Report if persistent. |

### Error response format (RFC 7807)

```json
{
  "code": "UNBALANCED_ENTRY",
  "message": "Journal entry does not balance: debits=1050, credits=1000",
  "field": null,
  "suggestion": "Adjust line amounts so total debits equal total credits"
}
```

Always check the `suggestion` field — it provides specific guidance for recovery.

### Common mistakes and fixes

**"Entry does not balance"**
- Double-check all amounts are in the smallest unit (cents, not dollars)
- Verify you haven't mixed up debit and credit on a line
- Sum all `debit_amount` values and all `credit_amount` values — they must be equal

**"Cannot post to control account"**
- The account has `has_subledger=true`
- Find or create a sub-account under it and post there instead

**"No open period for date"**
- The entry_date doesn't fall within any period, or the period is closed
- Create a new period covering that date, or change the entry date

**"Currency mismatch"**
- All accounts in a journal entry must share the same currency
- Check each account's `currency_id` before posting
