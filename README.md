<p align="center">
  <img src="frontend/static/logo.svg" alt="ClawCounting" width="200">
</p>

<h1 align="center">ClawCounting</h1>

<p align="center">Foundational double-entry bookkeeping engine for AI agents. Single binary + single SQLite database.</p>

Designed as a platform, not an end-user application -- it provides the primitives (currencies, accounts, journal entries, periods, balances, reports) that more complex systems are built on top of. Consuming systems -- whether AI agents, ERPs, invoicing systems, or custom business logic -- use the API or CLI to build domain-specific workflows.

## Features

- **Double-entry accounting** -- every journal entry must balance (total debits == total credits), with minimum 2 lines, all in the same currency
- **Immutable journal** -- journal entries are append-only; corrections via reversing entries only
- **Financial periods** -- fiscal year management with permanent close, automatic closing entries (revenue/expense zeroing into retained earnings)
- **Subledger support** -- control accounts with per-entity sub-accounts (AR/AP by customer/vendor)
- **Multi-currency** -- fiat (ISO 4217) and crypto (ERC-20, native coins) via CAIP-19 identifiers, with full wei-precision (i128 amounts)
- **Two interfaces** -- REST API (for agents and web UI) and CLI (for scripts, cron, admin) sharing the same service layer
- **Built-in web UI** -- SvelteKit SPA embedded in the binary
- **OpenAPI docs** -- auto-generated spec with Swagger UI at `/swagger-ui`
- **Agent Skill** -- [agentskills.io](https://agentskills.io) standard skill for teaching AI agents accounting workflows
- **Zero deployment overhead** -- no Docker, no external database servers, no runtime dependencies

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [pnpm](https://pnpm.io/) (for frontend development)

### Build and Run

```bash
# Clone and install
git clone https://github.com/johnkozan/clawcounting.git
cd clawcounting
cargo install --path .

# Initialize the database and start the server
clawcounting init
clawcounting serve
```

The server starts at `http://localhost:3000`. On first run, you'll be guided through setup (creating your first user) via the web UI.

For full documentation, see the [ClawCounting Docs](https://johnkozan.github.io/clawcounting/docs/).

## Agent Skill

ClawCounting includes an [Agent Skill](https://agentskills.io) in `skill/SKILL.md` that teaches AI agents how to use the accounting engine -- domain rules, workflows, and best practices. Supported by Claude Code, Cursor, VS Code/Copilot, and other agent platforms.

## License

MIT -- see [LICENSE](LICENSE) for details.
