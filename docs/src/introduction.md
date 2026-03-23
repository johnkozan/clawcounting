# Introduction

ClawCounting is a foundational double-entry bookkeeping engine designed for AI agents. It ships as a single Rust binary backed by a single SQLite database file.

It is a **platform, not an end-user application** -- it provides the primitives (currencies, accounts, journal entries, periods, balances, reports) that more complex systems are built on top of. Consuming systems -- whether AI agents, ERPs, invoicing systems, or custom business logic -- use the API or CLI to build domain-specific workflows (AR/AP, payroll, tax, reconciliation, approvals, etc.).

## Key Features

- **Double-entry accounting** -- every journal entry must balance (total debits == total credits), with minimum 2 lines, all in the same currency
- **Immutable journal** -- journal entries are append-only; corrections via reversing entries only
- **Financial periods** -- fiscal year management with permanent close, automatic closing entries
- **Subledger support** -- control accounts with per-entity sub-accounts (AR/AP by customer/vendor)
- **Multi-currency** -- fiat (ISO 4217) and crypto (ERC-20, native coins) via CAIP-19 identifiers, with full wei-precision (i128 amounts)
- **Two interfaces** -- REST API (for agents and web UI) and CLI (for scripts, cron, admin) sharing the same service layer
- **Built-in web UI** -- SvelteKit SPA embedded in the binary
- **OpenAPI docs** -- auto-generated spec with Swagger UI
- **Agent Skill** -- [agentskills.io](https://agentskills.io) standard skill for teaching AI agents accounting workflows
- **Zero deployment overhead** -- no Docker, no external database servers, no runtime dependencies

## Design Philosophy

- **Single-tenant** -- each deployment serves one tenant. Multi-tenant = separate instances.
- **Server and CLI share the same service layer** -- identical validation, business logic, and transactions regardless of how you interact with it.
- **The API enforces correctness** -- balanced entries, period rules, subledger constraints. The less computation delegated to the calling agent, the fewer errors.
- **Structured errors with recovery guidance** -- every error includes a `suggestion` field telling the caller how to fix it.
- **Trivially deployable** -- copy one binary, run it. The SQLite `.db` file is your entire database.

## Tech Stack

| Component | Choice |
|-----------|--------|
| Language | Rust |
| Web Framework | Axum |
| Database | SQLite 3 (rusqlite, bundled + i128_blob) |
| Connection Pool | deadpool-sqlite |
| Migrations | refinery (embedded SQL, forward-only) |
| Frontend | SvelteKit SPA (adapter-static) + Tailwind CSS + shadcn-svelte |
| API Docs | utoipa + swagger-ui |
| Auth | API keys (agents) + JWT (web users) |
| CLI | clap |
