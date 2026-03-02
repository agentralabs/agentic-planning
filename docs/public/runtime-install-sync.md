---
status: stable
---

# Runtime, Installation & Sync

Operations for installing, running, and managing AgenticPlanning instances.

## Installation

### From Source

```bash
git clone https://github.com/agentra/agentic-planning
cd agentic-planning
cargo build --release
# Binary at target/release/aplan
```

### Cargo Install

```bash
cargo install agentic-planning-cli
```

This places the `aplan` binary in `~/.cargo/bin/`. Ensure this is on your `PATH`.

### Verify

```bash
aplan version
```

## Runtime Detection

The CLI determines engine mode from the `--file` flag:

- **No `--file`** — In-memory mode. State exists only for the duration of the command. Each invocation starts fresh.
- **`--file project.aplan`** — File-backed mode. Loads existing state or creates a new file. Every mutation triggers an atomic save.

The `serve` command follows the same pattern:

```bash
aplan serve                          # In-memory MCP server
aplan serve --file project.aplan     # Persistent MCP server
```

## MCP Server Lifecycle

### Stdio Mode (Default)

```bash
aplan serve --mode stdio
```

The server communicates over stdin/stdout using Content-Length framed JSON-RPC 2.0 — the standard MCP transport protocol. Each message is prefixed with `Content-Length: <bytes>\r\n\r\n`.

The server handles three phases:
1. **Initialize** — Client sends `initialize` request. Server responds with capabilities (13 tools, resources, prompts).
2. **Operation** — Client sends `tools/call` requests. Server processes and responds.
3. **Shutdown** — Client sends `shutdown` or closes stdin. Server exits cleanly.

### HTTP Mode

```bash
aplan serve --mode http --port 3000
```

HTTP transport on the specified port. Same JSON-RPC 2.0 protocol over HTTP POST requests.

## Daemon Mode

Background monitoring for stale goals, at-risk commitments, and blocked progress:

```bash
# Start daemon with default 300-second interval
aplan daemon start --file project.aplan

# Custom interval
aplan daemon start --file project.aplan --interval_secs 60

# Stop daemon
aplan daemon stop
```

The daemon periodically:
- Checks for goals with rising neglect (no progress updates)
- Flags commitments approaching their deadlines
- Detects blocked goals that need attention
- Logs warnings to stderr

## Workspace Management

Workspaces isolate planning contexts within a single engine:

```bash
# Create a workspace
aplan workspace create --name "backend-v2"

# List workspaces
aplan workspace list

# Switch active workspace
aplan workspace switch --name "backend-v2"

# Compare planning state across workspaces
aplan workspace compare --a "backend-v2" --b "frontend-v2"
```

All goal, decision, and commitment operations apply to the active workspace.

## Multi-Instance Access

Multiple CLI or MCP instances can reference the same `.aplan` file. The engine uses atomic writes (write to `.tmp`, fsync, rename) so readers never see a partially written file.

**Concurrency model:**
- Each engine instance loads the full file on startup
- Writes are atomic — no partial reads possible
- No file-level locking — last writer wins
- For coordinated multi-agent access, use federation instead of shared files

**Best practice:** Use one MCP server instance per `.aplan` file. Multiple agents connect to the same server via MCP rather than opening the file independently.

## State Migration

The `.aplan` file is fully self-contained. To migrate planning state:

```bash
# Copy to another machine
scp project.aplan user@remote:/path/to/project.aplan

# Open on the remote machine
aplan --file /path/to/project.aplan goal list
```

The file contains all goals, decisions, commitments, dreams, federations, soul archives, and indexes. No external database or service required.

## Crash Recovery

If the process crashes during a save, the engine recovers automatically on next load:

| `.aplan` exists | `.aplan.tmp` exists | Action |
|:---:|:---:|--------|
| No | No | Create new empty file |
| No | Yes | Recover: rename `.tmp` to `.aplan`, then load |
| Yes | No | Normal load |
| Yes | Yes | Load `.aplan` (complete), delete stale `.tmp` |

No manual intervention required. The atomic write strategy ensures the `.aplan` file is always either the previous valid version or the new valid version.
