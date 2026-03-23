# Double-Entry Bookkeeping

Every financial transaction in ClawCounting is recorded as a journal entry with two or more lines. The fundamental rule:

> **Total debits must equal total credits** in every journal entry.

An unbalanced entry is rejected with error code `UNBALANCED_ENTRY`.

## Debits and Credits

| Account Type | Debit increases | Credit increases |
|-------------|----------------|-----------------|
| Asset | Balance goes up | Balance goes down |
| Expense | Balance goes up | Balance goes down |
| Liability | Balance goes down | Balance goes up |
| Equity | Balance goes down | Balance goes up |
| Revenue | Balance goes down | Balance goes up |

**Mnemonic**: Assets and Expenses are "debit-normal" -- they increase with debits. Everything else is "credit-normal."

## Account Types and Normal Balances

| Type | Normal Balance | Equation Side | Closed at Period End? |
|------|---------------|---------------|----------------------|
| `asset` | `debit` | Left (A) | No -- permanent |
| `liability` | `credit` | Right (L) | No -- permanent |
| `equity` | `credit` | Right (E) | No -- permanent |
| `revenue` | `credit` | Income Statement | Yes -- zeroed to retained earnings |
| `expense` | `debit` | Income Statement | Yes -- zeroed to retained earnings |

**The accounting equation**: Assets = Liabilities + Equity

ClawCounting verifies this equation in the balance sheet report (`is_balanced` field).

## Permanent vs Temporary Accounts

- **Permanent** (asset, liability, equity): Balances carry forward across periods.
- **Temporary** (revenue, expense): Balances are zeroed when a period is closed, with net income transferred to retained earnings.

## Examples

**Receive cash for services rendered:**
- Debit Cash (asset goes up)
- Credit Revenue (revenue goes up)

**Pay rent:**
- Debit Rent Expense (expense goes up)
- Credit Cash (asset goes down)

**Take out a loan:**
- Debit Cash (asset goes up)
- Credit Loan Payable (liability goes up)

**Multi-line entry -- payroll with tax withholding:**
- Debit Wages Expense $5,000
- Credit Tax Payable $750
- Credit Cash $4,250

Total debits ($5,000) = total credits ($750 + $4,250). The entry balances.

## Validation Rules

ClawCounting enforces these rules on every journal entry:

1. **Balanced** -- total debits must equal total credits
2. **Minimum 2 lines** -- at least one debit and one credit
3. **Same currency** -- all lines must reference accounts with the same currency
4. **Open period** -- the entry date must fall within an open financial period
5. **Active accounts** -- all referenced accounts must be active
6. **No control accounts** -- cannot post directly to accounts with `has_subledger=true`
7. **Non-negative amounts** -- each line has either a debit or credit amount, not both, and it must be positive
