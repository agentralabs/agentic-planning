---
status: stable
---

# Installation

AgenticPlanning can be installed through four methods. Choose the one that fits your workflow.

## Cargo Install (Recommended for Rust users)

Requires Rust 1.75+:

```bash
cargo install agentic-planning-cli
```

For the standalone MCP server binary:

```bash
cargo install agentic-planning-mcp
```

This builds and installs the `aplan` binary to `~/.cargo/bin/`. The binary includes the CLI, MCP server (`aplan serve`), and daemon mode. The standalone `agentic-planning-mcp` binary is useful for deployments that only need the MCP server.

## Pre-built Binary

Platform-detected installer:

```bash
curl -fsSL https://agentralabs.tech/install/planning | bash
```

The installer detects your platform (macOS arm64/x86_64, Linux x86_64), downloads the binary, places it in `~/.local/bin/`, and adds it to PATH if needed.

Verify:

```bash
aplan version
```

## npm Package (WASM)

For Node.js or browser usage:

```bash
npm install @agenticamem/planning
```

```javascript
const { PlanningEngine } = require('@agenticamem/planning');

const engine = PlanningEngine.inMemory();
const goalId = engine.createGoal({
  title: "Ship feature",
  intention: "Deliver value to users",
  priority: "high"
});
```

## pip Package (Python FFI)

For Python usage via ctypes:

```bash
pip install agentic-planning
```

```python
from agentic_planning import PlanningEngine

engine = PlanningEngine.in_memory()
goal_id = engine.create_goal(
    title="Ship feature",
    intention="Deliver value to users",
    priority="high"
)
```

The pip package bundles the native shared library and provides Python bindings over the 27 FFI functions.

## From Source

```bash
git clone https://github.com/anthropics/agentic-planning.git
cd agentic-planning
cargo build --release
```

Binaries are placed in `target/release/`:

| Binary | Purpose |
|--------|---------|
| `aplan` | CLI + MCP server + daemon |
| `agentic-planning-mcp` | Standalone MCP server |

## MCP Server Setup

To use AgenticPlanning with Claude Code, add the MCP server configuration to your settings:

**Claude Code** (`~/.claude/settings.json`):

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

**With a persistent state file:**

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

The MCP server communicates over stdio using Content-Length framed JSON-RPC 2.0 — the standard MCP transport protocol.

## Verification

After installation, confirm everything works:

```bash
# Check version
aplan version

# Check engine status
aplan status

# Create a test goal
aplan goal create "Test goal" --intention "Verify installation"

# List goals
aplan goal list
```

## Upgrading

- **Cargo:** `cargo install agentic-planning-cli` (rebuilds latest)
- **Binary:** Re-run the curl installer
- **npm:** `npm update @agenticamem/planning`
- **pip:** `pip install --upgrade agentic-planning`

The `.aplan` file format is version-safe across minor releases. The header version field ensures older files are loaded correctly by newer engines.
