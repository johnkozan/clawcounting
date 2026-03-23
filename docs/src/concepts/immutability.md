# Immutability & Corrections

ClawCounting enforces strict immutability on accounting records to maintain a complete audit trail.

## What Cannot Be Changed

- **Journal entries** -- no UPDATE, no DELETE. Once posted, the entry exists permanently.
- **Journal entry lines** -- append-only.
- **Closed periods** -- cannot be reopened.
- **Currency code, caip19_id, and asset_scale** -- immutable after creation.
- **Account type, normal_balance, and currency_id** -- immutable after creation.

## What Can Be Changed

- Currency `name` and `symbol`
- Account `name`, `is_active`, and `xbrl_tag`
- User `name`, `permissions`, and `is_active`
- Settings values

## How to Correct Mistakes

Since journal entries cannot be edited or deleted, corrections are made through **reversing entries**:

1. **Reverse** the incorrect entry -- this creates a new entry with all debits and credits swapped
2. **Post** a new entry with the correct amounts

Both the original error and the correction remain in the ledger, providing a complete audit trail.

### Example

```bash
# 1. Reverse the wrong entry
clawcounting journal-entries reverse <wrong-entry-id> --api-key $API_KEY --json

# 2. Post the corrected entry
clawcounting journal-entries create --file corrected-entry.json --api-key $API_KEY --json
```

## Reversal Rules

- The reversal date must fall within an open period
- A reversal cannot itself be reversed
- The original entry is not modified -- it remains in the ledger
- After reversal, the net effect on all accounts is zero
- The reversal entry has `is_reversal=true` and `reverses_id` pointing to the original

## Why Immutability?

This design follows standard accounting practice:

- **Audit trail** -- every change is visible and traceable
- **Data integrity** -- no risk of accidentally or maliciously altering historical records
- **Regulatory compliance** -- many jurisdictions require immutable financial records
- **Simplicity** -- the balance trigger only needs to handle INSERTs, never UPDATEs or DELETEs
