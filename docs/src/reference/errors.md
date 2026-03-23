# Error Codes

All errors follow [RFC 7807](https://tools.ietf.org/html/rfc7807) (Problem Details) and include a `suggestion` field with recovery guidance.

## Error Response Format

```json
{
  "code": "UNBALANCED_ENTRY",
  "message": "Journal entry does not balance: debits=1050, credits=1000",
  "field": null,
  "suggestion": "Adjust line amounts so total debits equal total credits"
}
```

Always check the `suggestion` field -- it provides specific guidance for recovery.

## Error Code Reference

| Code | HTTP Status | Meaning | Recovery |
|------|------------|---------|----------|
| `VALIDATION_ERROR` | 400 | Field validation failed | Read `field` and `suggestion` -- fix the specific field |
| `UNBALANCED_ENTRY` | 400 | Total debits != total credits | Check line amounts. Ensure they sum to equal values. |
| `PERIOD_CLOSED` | 409 | Period is closed | Post to a different open period |
| `NOT_FOUND` | 404 | Resource doesn't exist | Verify the ID. List resources to find the correct one. |
| `UNAUTHORIZED` | 401 | Missing or invalid auth | Check Authorization header. Re-login or use valid API key. |
| `FORBIDDEN` | 403 | Insufficient permissions | Check user permissions. Request access from admin. |
| `DATABASE_ERROR` | 500 | Database failure | Retry. If persistent, check disk space and database integrity. |
| `INTERNAL_ERROR` | 500 | Unexpected error | Retry. Report if persistent. |

## Common Mistakes and Fixes

### "Entry does not balance"

- Double-check all amounts are in the smallest unit (cents, not dollars)
- Verify you haven't mixed up debit and credit on a line
- Sum all `debit_amount` values and all `credit_amount` values -- they must be equal

### "Cannot post to control account"

- The account has `has_subledger=true`
- Find or create a sub-account under it and post there instead
- List sub-accounts: `GET /api/v1/accounts/{id}/sub-accounts`

### "No open period for date"

- The `entry_date` doesn't fall within any period, or the matching period is closed
- Create a new period covering that date, or change the entry date
- List periods: `clawcounting periods list --json`

### "Currency mismatch"

- All accounts in a journal entry must share the same currency
- Check each account's `currency_id` before posting

### "Retained earnings account not configured"

- Period close requires `retained_earnings_account_id` in settings
- Set it: `clawcounting settings set retained-earnings-account <id>`
- The account must be an equity-type account
