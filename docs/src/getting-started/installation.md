# Installation

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [pnpm](https://pnpm.io/) (only needed for frontend development)

## Build from Source

```bash
# Clone and install
git clone https://github.com/johnkozan/clawcounting.git
cd clawcounting
cargo install --path .

# Verify
clawcounting --version
```

## Quick Start

```bash
# Initialize the database and start the server
clawcounting init
clawcounting serve
```

The server starts at `http://localhost:3000`. On first run, you'll be guided through setup (creating your first user) via the web UI.

## Configuration

ClawCounting works out of the box with sensible defaults -- no configuration file needed. You can optionally override settings via environment variables or a `.env` file:

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAWCOUNTING_DB` | `./clawcounting.db` | SQLite database file path |
| `CLAWCOUNTING_HOST` | `127.0.0.1` | Address to bind the HTTP server to. Set to `0.0.0.0` to listen on all interfaces. |
| `CLAWCOUNTING_PORT` | `3000` | HTTP server port |
| `CLAWCOUNTING_JWT_SECRET` | Auto-generated | JWT signing secret. Auto-generated and stored in DB if not set. |
| `CLAWCOUNTING_API_KEY` | -- | API key for CLI write operations. Alternative to `--api-key` flag. |

A `.env.example` file is included in the repository for reference.

## What Gets Created

Running `clawcounting init` creates:
- The SQLite database file (default: `./clawcounting.db`)
- All tables, indexes, triggers, and migrations
- A JWT secret (stored in the settings table)

Running `clawcounting serve` starts:
- REST API at `/api/v1/*`
- Web UI at `/`
- Swagger API docs at `/swagger-ui/`
- Health check at `/health`
