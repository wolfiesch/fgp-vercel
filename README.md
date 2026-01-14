# FGP Vercel Daemon

Fast Vercel operations via FGP daemon. Manage projects, deployments, and logs without MCP cold-start overhead.

## Installation

```bash
git clone https://github.com/wolfiesch/fgp-vercel.git
cd fgp-vercel
cargo build --release
```

**Requirements:**
- Rust 1.70+
- Vercel access token (`VERCEL_TOKEN` env var)

## Quick Start

```bash
# Set your Vercel token
export VERCEL_TOKEN="xxxxx"

# Start the daemon
./target/release/fgp-vercel start

# List projects
fgp call vercel.projects

# Get project details
fgp call vercel.project '{"project_id": "my-project"}'

# List deployments
fgp call vercel.deployments '{"project_id": "my-project", "limit": 5}'

# Get deployment logs
fgp call vercel.logs '{"deployment_id": "dpl_xxxxx"}'

# Stop daemon
./target/release/fgp-vercel stop
```

## Available Methods

| Method | Params | Description |
|--------|--------|-------------|
| `vercel.projects` | `limit` (default: 20) | List all projects |
| `vercel.project` | `project_id` (required) | Get project details |
| `vercel.deployments` | `project_id`, `limit` | List deployments |
| `vercel.deployment` | `deployment_id` (required) | Get deployment details |
| `vercel.logs` | `deployment_id` (required) | Get deployment logs/events |
| `vercel.user` | - | Get current user info |

## FGP Protocol

Socket: `~/.fgp/services/vercel/daemon.sock`

**Request:**
```json
{"id": "uuid", "v": 1, "method": "vercel.deployments", "params": {"project_id": "my-app", "limit": 5}}
```

**Response:**
```json
{"id": "uuid", "ok": true, "result": {"deployments": [...], "count": 5}}
```

## Why FGP?

| Operation | FGP Daemon | MCP stdio | Speedup |
|-----------|------------|-----------|---------|
| List projects | ~200ms | ~2,500ms | **12x** |
| Get logs | ~180ms | ~2,400ms | **13x** |

FGP keeps the API connection warm, eliminating cold-start overhead.

## Use Cases

- **Deployment monitoring**: Quick status checks during CI/CD
- **Log debugging**: Fast access to deployment events
- **Project management**: List and query projects programmatically
- **AI agents**: Integrate deployment status into agent workflows

## License

MIT
