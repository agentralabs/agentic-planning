---
status: stable
---

# Configuration

AgenticPlanning is configured through CLI flags, MCP server settings, and engine mode selection. There is no configuration file — behavior is controlled entirely through arguments and code.

## CLI Global Options

Every `aplan` command accepts these global flags:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--file <PATH>` | Path | None | State file path. If omitted, uses in-memory engine |
| `--format <FORMAT>` | text/json/table | text | Output format |
| `--json` | Flag | false | Shorthand for `--format json` |
| `--verbose` | Flag | false | Enable verbose output |

Examples:

```bash
# In-memory (ephemeral, no persistence)
aplan goal create "Ship v1" --intention "Deliver"

# File-backed (persistent)
aplan --file project.aplan goal create "Ship v1" --intention "Deliver"

# JSON output
aplan --json goal list

# Table output
aplan --format table goal list
```

## Engine Modes

### In-Memory Mode

```rust
let engine = PlanningEngine::in_memory();
```

State exists only in RAM. No file I/O. Used for:
- Tests and benchmarks
- Ephemeral single-session planning
- WASM/browser environments
- Piped CLI workflows where persistence isn't needed

### File-Backed Mode

```rust
let engine = PlanningEngine::open("project.aplan")?;
```

Loads existing state or creates a new file. Every mutation triggers an atomic save. Used for:
- Multi-session projects
- MCP server with persistent state
- Team collaboration via shared `.aplan` files

`open()` handles crash recovery automatically — if only a `.aplan.tmp` exists, it's promoted to the primary file.

## MCP Server Configuration

### Stdio Mode (Default)

```bash
aplan serve
aplan serve --mode stdio
```

Communicates over stdin/stdout using Content-Length framed JSON-RPC 2.0 — the standard MCP transport protocol.

### With Persistent State

```bash
aplan serve --file project.aplan
```

All 13 MCP tools operate on the same file-backed engine. State persists across tool calls.

### HTTP Mode

```bash
aplan serve --mode http --port 3000
```

Planned HTTP transport on the specified port. Default port: 3000.

### Claude Code Integration

Add to `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "agentic-planning": {
      "command": "aplan",
      "args": ["serve"]
    }
  }
}
```

With a persistent state file:

```json
{
  "mcpServers": {
    "agentic-planning": {
      "command": "aplan",
      "args": ["serve", "--file", "/path/to/project.aplan"]
    }
  }
}
```

## Serve Command Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--mode <MODE>` | stdio/http | stdio | Transport mode |
| `--port <PORT>` | u16 | 3000 | HTTP port (only for http mode) |
| `--file <PATH>` | Path | None | Persistent state file (global flag) |

## Daemon Settings

Background monitoring mode:

```bash
aplan daemon start
aplan daemon start --interval_secs 60
aplan daemon stop
```

The daemon periodically checks for stale goals (rising neglect), at-risk commitments (approaching deadlines), and blocked progress. Logs go to stderr.

## Bridge Configuration

Bridges are configured programmatically via the `BridgeConfig` struct:

```rust
use agentic_planning_bridges::*;

// Default: all NoOp bridges (standalone mode)
let config = BridgeConfig::default();

// With a real memory bridge
let config = BridgeConfig {
    memory: Some(Box::new(MyMemoryBridge::new())),
    identity: Some(Box::new(MyIdentityBridge::new())),
    ..Default::default()
};
```

Each bridge is optional. `None` falls back to the `NoOp` implementation which returns neutral values (empty lists, zero scores, auto-approved results).

## Output Formats

Three output formats are available across all CLI commands:

**Text** (default) — Human-readable, colored terminal output:
```
Goal: a1b2c3d4
  Title: Ship v1.0
  Status: Active
  Priority: High
```

**JSON** — Machine-parseable, suitable for piping:
```json
{"id": "a1b2c3d4", "title": "Ship v1.0", "status": "Active"}
```

**Table** — Aligned columns for listing commands:
```
ID          TITLE       STATUS   PRIORITY
a1b2c3d4    Ship v1.0   Active   High
```
