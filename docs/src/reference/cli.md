# CLI Reference

ClawCounting's CLI provides direct access to all accounting operations. It connects directly to the SQLite database -- no server needed.

## Global Options

| Option | Description |
|--------|-------------|
| `--db <path>` | Database file path (default: `./clawcounting.db` or `CLAWCOUNTING_DB` env var) |
| `--json` | Machine-readable JSON output |
| `--api-key <key>` | API key for write operations (or set `CLAWCOUNTING_API_KEY` env var) |
| `--version` | Print version |
| `--help` | Print help |

## Commands

### Database

```bash
clawcounting init                      # Initialize database (create file, run migrations)
clawcounting serve                     # Start HTTP server + web UI
```

### Currencies

```bash
clawcounting currencies list                              # List all currencies
clawcounting currencies create-fiat USD                   # Create fiat from ISO 4217 code
clawcounting currencies create \
  --code ETH \
  --name "Ether" \
  --symbol "Ξ" \
  --asset-scale 18 \
  --type crypto \
  --caip19 "eip155:1/slip44:60"                           # Create custom currency
```

### Accounts

```bash
clawcounting accounts list                                # List all accounts
clawcounting accounts list --type asset                   # Filter by type
clawcounting accounts get <id>                            # Get account details + balance
clawcounting accounts create \
  --name "Cash" \
  --currency <currency-id> \
  --type asset \
  --normal-balance debit \
  --number 1000                                           # Create account
clawcounting accounts create \
  --name "AR - Acme" \
  --number 1200-001 \
  --parent <control-account-id> \
  --entity "acme-corp"                                    # Create sub-account
```

### Journal Entries

```bash
clawcounting journal-entries list                         # List entries
clawcounting journal-entries list --period <period-id>    # Filter by period
clawcounting journal-entries get <id>                     # Get entry with lines
clawcounting journal-entries create --file entry.json     # Create from JSON file
clawcounting journal-entries reverse <id>                 # Reverse an entry
clawcounting journal-entries reverse <id> --date 2026-04-01  # Reverse with new date
```

### Financial Periods

```bash
clawcounting periods list                                 # List all periods
clawcounting periods get <id>                             # Get period details
clawcounting periods create \
  --name "FY2026" \
  --start 2026-01-01 \
  --end 2026-12-31                                        # Create period
clawcounting periods close <id> --preview                 # Preview closing entry
clawcounting periods close <id>                           # Close permanently
```

### Reports

```bash
clawcounting reports trial-balance                        # All-time trial balance
clawcounting reports trial-balance --period <id>          # Period-specific
clawcounting reports balance-sheet                        # Current balance sheet
clawcounting reports balance-sheet --as-of 2026-06-30     # As of date
clawcounting reports balance-sheet --period <id>          # Through end of period
clawcounting reports income-statement --period <id>       # Income for period
clawcounting reports general-ledger \
  --account <id> \
  --start 2026-01-01 \
  --end 2026-03-31                                        # Account detail
```

### Users

```bash
clawcounting users list                                   # List all users
clawcounting users get <id>                               # Get user details
clawcounting users create \
  --name "Admin" \
  --email "admin@example.com" \
  --password "secure"                                     # Create human user
clawcounting users create-service-account \
  --name "AI Agent"                                       # Create service account (returns API key)
```

### Settings

```bash
clawcounting settings get                                 # Show all settings
clawcounting settings set retained-earnings-account <id>  # Set retained earnings account
```

## API Key Requirements

Commands that create accounting records require an API key for user attribution:

| Command | API Key Required |
|---------|:---:|
| `currencies list/create` | No |
| `accounts list/create/get` | No |
| `periods list/create/get` | No |
| `reports *` | No |
| `users list/create/get` | No |
| `settings get/set` | No |
| `journal-entries create` | **Yes** |
| `journal-entries reverse` | **Yes** |
| `periods close` | **Yes** |

## CLI / API Equivalents

| CLI Command | API Endpoint |
|-------------|-------------|
| `currencies create ...` | `POST /api/v1/currencies` |
| `currencies list` | `GET /api/v1/currencies` |
| `currencies create-fiat USD` | N/A (CLI convenience) |
| `accounts create ...` | `POST /api/v1/accounts` |
| `accounts list` | `GET /api/v1/accounts` |
| `journal-entries create --file ...` | `POST /api/v1/journal-entries` |
| `journal-entries reverse <id>` | `POST /api/v1/journal-entries/{id}/reverse` |
| `periods create ...` | `POST /api/v1/periods` |
| `periods close <id>` | `POST /api/v1/periods/{id}/close` |
| `reports trial-balance` | `GET /api/v1/reports/trial-balance` |
| `reports balance-sheet` | `GET /api/v1/reports/balance-sheet` |
| `reports income-statement` | `GET /api/v1/reports/income-statement` |
| `reports general-ledger` | `GET /api/v1/reports/general-ledger` |
| `settings set ...` | `PATCH /api/v1/settings` |
