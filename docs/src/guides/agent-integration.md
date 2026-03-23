# Agent Integration

ClawCounting is designed for AI agent consumption. This guide covers how to integrate an AI agent with ClawCounting.

## Agent Skill

ClawCounting includes an [Agent Skill](https://agentskills.io) in `skill/SKILL.md`. The skill teaches AI agents how to use the accounting engine -- domain rules, workflows, and best practices.

Supported platforms: Claude Code, Claude.ai, VS Code/Copilot, Cursor, OpenAI Codex, Gemini CLI, JetBrains Junie, and others.

The skill follows a progressive disclosure model:
1. **Metadata** (~100 tokens) -- name and description, loaded at startup for activation matching
2. **Instructions** (<5,000 tokens) -- full SKILL.md body, loaded when the agent does accounting work
3. **References** (on demand) -- detailed guides loaded only when specific procedures are needed

## CLI Interface (Recommended)

The CLI is the recommended interface for AI agents. It connects directly to the SQLite database with no server needed.

```bash
# Always use --json for machine-readable output
clawcounting currencies list --json
clawcounting accounts list --json
clawcounting reports trial-balance --json
```

### Authentication for Write Operations

Commands that create accounting records (journal entries, reversals, period close) require an API key:

```bash
# Via environment variable (recommended)
export CLAWCOUNTING_API_KEY=tsk_...
clawcounting journal-entries create --file entry.json --json

# Via flag
clawcounting journal-entries create --file entry.json --api-key tsk_... --json
```

Admin commands (user/currency/account/period creation, reports, settings) work without an API key.

## HTTP API

For agents that communicate over HTTP:

```bash
curl -H "Authorization: Bearer tsk_..." \
  http://localhost:3000/api/v1/accounts --json
```

See [API Reference](../reference/api.md) for all endpoints.

### Starting the Server in the Background

When an agent needs to start the server as part of a workflow:

```bash
# Start server in background
nohup clawcounting serve > /tmp/clawcounting.log 2>&1 &
echo $! > /tmp/clawcounting.pid

# Wait for readiness
until curl -sf http://localhost:3000/health > /dev/null 2>&1; do sleep 0.5; done

# Server is now accepting requests
```

To stop:
```bash
kill "$(cat /tmp/clawcounting.pid)"
```

## Error Handling

Errors follow RFC 7807 and include a `suggestion` field with recovery guidance:

```json
{
  "code": "PERIOD_CLOSED",
  "message": "Period FY2025 is closed",
  "field": null,
  "suggestion": "Post to period FY2026 which is currently open"
}
```

Always read the `suggestion` before retrying. See [Error Codes](../reference/errors.md) for the full list.

## Response Envelope

Single resource:
```json
{ "data": { ... } }
```

List (cursor-paginated):
```json
{ "data": [...], "has_more": true, "next_cursor": "..." }
```

Pagination: `?limit=50&cursor=<next_cursor>`. Default limit 50, max 200.

## OpenAPI Schema

The full OpenAPI spec is available at `/swagger-ui` when the server is running. Agents can use the schema for endpoint discovery and parameter validation.

## Typical Agent Workflow

1. **Check setup** -- verify currencies, accounts, and periods exist
2. **Post entries** -- create journal entries for transactions
3. **Query state** -- check balances, run reports
4. **Handle errors** -- read error suggestions, adjust and retry
5. **Period management** -- preview and close periods when appropriate
