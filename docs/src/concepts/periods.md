# Financial Periods

Financial periods define the date ranges for accounting activity. Every journal entry must fall within an open period.

## Rules

- `start_date` must be before `end_date`
- Periods must not overlap
- Periods can be any duration (day, month, quarter, year)
- Gaps between periods are allowed
- Multiple periods can be open simultaneously

## Open vs Closed

- **Open**: Accepts new journal entries within its date range.
- **Closed**: Permanently sealed. No new entries, no reopening.

## Entry Date Validation

When posting a journal entry, ClawCounting:

1. Looks up which period contains the `entry_date`
2. If no period covers that date -- error (`NO_OPEN_PERIOD`)
3. If the matching period is closed -- error (`PERIOD_CLOSED`)
4. The entry is linked to that period via `period_id`

## Period Close

Closing a period is a permanent operation that:

1. Debits each **revenue** account to zero its balance for the period
2. Credits each **expense** account to zero its balance for the period
3. Posts the net income (or net loss) to the retained earnings account
4. Creates an automatic closing journal entry linked to the period
5. Marks the period as closed with a timestamp

### Prerequisites

- `retained_earnings_account_id` must be set in settings
- The retained earnings account must be an equity-type account
- The period must be open

### Workflow

Always preview first:

```bash
# Preview -- shows the closing entry without committing
clawcounting periods close <period-id> --preview --api-key $API_KEY --json

# Close -- permanent
clawcounting periods close <period-id> --api-key $API_KEY --json
```

### After Closing

- No new entries can be posted to the closed period
- The period cannot be reopened
- Errors in closed periods are corrected via adjusting entries in an open period
- Asset, liability, and equity balances carry forward naturally (they are permanent accounts)

## Examples

### Annual periods

```bash
clawcounting periods create --name "FY2026" --start 2026-01-01 --end 2026-12-31 --json
```

### Quarterly periods

```bash
clawcounting periods create --name "Q1 2026" --start 2026-01-01 --end 2026-03-31 --json
clawcounting periods create --name "Q2 2026" --start 2026-04-01 --end 2026-06-30 --json
clawcounting periods create --name "Q3 2026" --start 2026-07-01 --end 2026-09-30 --json
clawcounting periods create --name "Q4 2026" --start 2026-10-01 --end 2026-12-31 --json
```
