# Architecture

## Overview

```
Browser / Swagger UI / AI Agents (HTTP)
            |
    clawcounting serve (port 3000)
      /api/v1/...  -> API routes
      /swagger-ui  -> OpenAPI docs
      /*           -> SvelteKit SPA
            |
      services/ layer (shared business logic)
            |
AI Agents / Scripts (CLI)
            |
        SQLite (WAL mode, single .db file)
```

Both the HTTP server and CLI share the same `services/` layer -- identical validation, business logic, and transaction handling. The only difference is the entry point (HTTP handlers vs CLI commands) and output format (JSON vs text tables).

## Single-Tenant Design

Strictly single-tenant. Each deployment (binary + `.db` file) serves one tenant. Multi-tenant is achieved by running separate instances, each with its own database file. This eliminates org_id from all tables, simplifies every query, and removes org-scoping from routes.

## SQLite Configuration

- **WAL mode** (Write-Ahead Logging) -- allows concurrent readers during writes
- **`synchronous = NORMAL`** -- good durability/performance balance
- **`foreign_keys = ON`** -- enforce referential integrity
- **`busy_timeout = 5000`** -- wait up to 5 seconds for write lock

## Connection Strategy

### Server Mode (deadpool-sqlite)

- **Write pool (size 1)** -- serializes all writes within the server process. Handlers calling `write_pool.get().await` queue up for the one connection.
- **Read pool (size 2-4)** -- multiple read-only connections for concurrent queries.
- All DB work runs inside `interact()` closures on blocking threads -- Tokio async workers are never blocked.

This mirrors SQLite's "single writer, multiple readers" concurrency model under WAL mode.

### CLI Mode (direct rusqlite)

Opens a single `rusqlite::Connection` directly -- no pool needed for a short-lived process.

### Cross-Process (server + CLI simultaneously)

Both processes may write to the same `.db` file. SQLite's file-level locking handles this. `busy_timeout = 5000` on every connection gives a 5-second window for lock contention. In practice, write transactions are short (milliseconds), so collisions are rare.

## Amount Storage

All monetary amounts are stored as **i128 integers** in 16-byte BLOBs. The encoding (MSB-flipped big-endian) makes `memcmp()` produce correct signed ordering, so SQLite's `ORDER BY`, `<`, `>`, `MIN`, `MAX` all work correctly.

Custom SQLite functions are registered on every connection:
- **`sum_i128(column)`** -- aggregate equivalent to `SUM()` for i128 BLOBs
- **`i128_add(a, b)`** -- scalar addition of two i128 BLOBs
- **`i128_to_text(column)`** -- converts BLOB to decimal string for debugging

## Balance Updates

A SQLite trigger (`update_balance_on_insert`) on `journal_entry_lines` automatically maintains per-period `account_balances` using `i128_add()`. The trigger fires within the same transaction as the journal entry insert -- no application-layer read-modify-write needed.

```sql
CREATE TRIGGER update_balance_on_insert
AFTER INSERT ON journal_entry_lines
BEGIN
    INSERT INTO account_balances (account_id, period_id, total_debits, total_credits)
    VALUES (
        NEW.account_id,
        (SELECT period_id FROM journal_entries WHERE id = NEW.journal_entry_id),
        NEW.debit_amount,
        NEW.credit_amount
    )
    ON CONFLICT(account_id, period_id) DO UPDATE SET
        total_debits = i128_add(total_debits, NEW.debit_amount),
        total_credits = i128_add(total_credits, NEW.credit_amount);
END;
```

## Transaction Safety

All write operations use `BEGIN IMMEDIATE` to acquire the write lock at transaction start (not deferred until the first write). This prevents race conditions -- for example, two concurrent period close requests both reading the period as open before either commits.

## Project Structure

```
src/
  main.rs              # Entrypoint, clap dispatch
  config.rs            # DB path, port, env vars (CLAWCOUNTING_*)
  app_state.rs         # Shared state (connection pools)
  error.rs             # AppError, RFC 7807 responses
  router.rs            # Route assembly
  db/
    connection.rs      # SQLite setup (WAL, pragmas, custom functions)
    i128_funcs.rs      # Custom SQLite functions
    pool.rs            # deadpool-sqlite pools
    migrations.rs      # refinery embedded migrations
  middleware/auth.rs   # API key + JWT validation
  models/              # Request/response structs
  handlers/            # Axum route handlers
  services/            # Business logic (shared between HTTP and CLI)
  cli/                 # CLI command handlers
migrations/            # SQL migration files
skill/                 # Agent Skill (agentskills.io standard)
frontend/              # SvelteKit SPA
tests/                 # Integration tests
```

## Startup Flow

1. Open a raw `rusqlite::Connection`
2. Set pragmas (WAL, foreign keys, synchronous, busy_timeout)
3. Register custom SQLite functions (`i128_add`, `sum_i128`, `i128_to_text`)
4. Run `refinery` embedded migrations
5. Branch:
   - **Server**: Close bootstrap connection, create deadpool pools (with custom functions registered on each connection via `post_create` hook), start Axum server
   - **CLI**: Keep the connection, execute the requested command, exit
