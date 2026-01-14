# FGP Neon Daemon

Fast Neon Postgres operations via FGP daemon. Query databases, manage branches, and explore schemas without MCP cold-start overhead.

## Installation

```bash
git clone https://github.com/wolfiesch/fgp-neon.git
cd fgp-neon
cargo build --release
```

**Requirements:**
- Rust 1.70+
- Neon API key (`NEON_API_KEY` env var)
- Neon org ID (`NEON_ORG_ID` env var)

## Quick Start

```bash
# Set your Neon credentials
export NEON_API_KEY="neon_api_xxxxx"
export NEON_ORG_ID="org-xxxxx"

# Start the daemon
./target/release/fgp-neon start

# List projects
fgp call neon.projects

# List branches
fgp call neon.branches '{"project_id": "proj-xxxxx"}'

# Run SQL query
fgp call neon.sql '{"project_id": "proj-xxxxx", "branch_id": "br-xxxxx", "query": "SELECT * FROM users LIMIT 5"}'

# Stop daemon
./target/release/fgp-neon stop
```

## Available Methods

| Method | Params | Description |
|--------|--------|-------------|
| `neon.projects` | `limit` (default: 10) | List all projects |
| `neon.project` | `project_id` (required) | Get project details |
| `neon.branches` | `project_id` (required) | List branches for a project |
| `neon.databases` | `project_id`, `branch_id` (required) | List databases |
| `neon.tables` | `project_id`, `branch_id`, `database` | List tables |
| `neon.schema` | `project_id`, `branch_id`, `database`, `table` | Get table schema |
| `neon.sql` | `project_id`, `branch_id`, `database`, `query` | Run SQL query |
| `neon.user` | - | Get current user info |

## FGP Protocol

Socket: `~/.fgp/services/neon/daemon.sock`

**Request:**
```json
{"id": "uuid", "v": 1, "method": "neon.sql", "params": {"project_id": "proj-xxx", "branch_id": "br-xxx", "query": "SELECT 1"}}
```

**Response:**
```json
{"id": "uuid", "ok": true, "result": {"rows": [{"?column?": 1}]}}
```

## Why FGP?

| Operation | FGP Daemon | MCP stdio | Speedup |
|-----------|------------|-----------|---------|
| List projects | ~180ms | ~2,400ms | **13x** |
| Run SQL | ~120ms | ~2,400ms | **20x** |

FGP keeps the API connection warm and reuses auth tokens.

## Use Cases

- **AI agents**: Fast database queries for RAG pipelines
- **Schema exploration**: Quick table/column lookups
- **Branch management**: Create/switch branches programmatically
- **Data validation**: Run checks without connection overhead

## License

MIT
