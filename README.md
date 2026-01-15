# FGP Neon Daemon

Fast Neon Postgres operations via FGP daemon. Query databases, manage branches, and explore schemas without MCP cold-start overhead.

## Installation

```bash
git clone https://github.com/fast-gateway-protocol/neon.git
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

## Troubleshooting

### Invalid API Key

**Symptom:** Requests fail with 401 or "unauthorized"

**Solutions:**
1. Verify key is set: `echo $NEON_API_KEY`
2. Check key format: should start with `neon_api_`
3. Generate new key at https://console.neon.tech/app/settings/api-keys

### Project Not Found

**Symptom:** "Project not found" for existing project

**Check:**
1. Project ID is correct (format: `proj-xxxxx`)
2. API key has access to the project
3. List projects first: `fgp call neon.projects`

### Branch Not Found

**Symptom:** "Branch not found" when querying

**Check:**
1. Branch ID format: `br-xxxxx`
2. Branch belongs to specified project
3. List branches: `fgp call neon.branches '{"project_id": "proj-xxx"}'`

### SQL Query Errors

**Symptom:** Queries fail with syntax or permission errors

**Check:**
1. SQL syntax is valid for Postgres
2. Table/column names are correct
3. User has SELECT permissions on target tables
4. Database name is specified if not using default

### Connection Timeout

**Symptom:** Queries hang or timeout

**Solutions:**
1. Neon databases auto-suspend after inactivity
2. First query may take 1-2s to wake the database
3. Check Neon status: https://neon.tech/status
4. Verify branch is not suspended in console

### Empty Results

**Symptom:** Queries return empty when data exists

**Check:**
1. Correct database specified (default is `neondb`)
2. Schema is correct (default is `public`)
3. Table has data: `SELECT COUNT(*) FROM table_name`

### Connection Refused

**Symptom:** "Connection refused" when calling daemon

**Solution:**
```bash
# Check daemon is running
pgrep -f fgp-neon

# Restart daemon
./target/release/fgp-neon stop
export NEON_API_KEY="neon_api_xxxxx"
./target/release/fgp-neon start

# Verify socket
ls ~/.fgp/services/neon/daemon.sock
```

### Rate Limiting

**Symptom:** 429 errors on bulk operations

**Solutions:**
1. Neon has API rate limits
2. Add delays between rapid queries
3. Use connection pooling for high-frequency access

## License

MIT
