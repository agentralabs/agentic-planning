# SPEC-11: Guides & Playbooks Documentation

**Priority:** P2
**Files:** `docs/public/integration-guide.md`, `docs/public/playbooks-agent-integration.md`, `docs/public/runtime-install-sync.md`
**Estimated changes:** ~350 lines total

## Problem

Agent builders have no documentation on how to integrate AgenticPlanning into their systems. There's no guide for using the Rust API, Python FFI, or MCP tools in agent workflows. The playbooks file is empty — no system prompt templates, no agent patterns, no multi-agent examples.

## Requirements

### R1: integration-guide.md (~150 lines)

Deep integration patterns for each surface:

- **Rust API integration** — Add dependency, create engine, basic CRUD workflow with code example (create goal → track progress → complete), error handling pattern with PlanningError
- **Python FFI binding** — Load shared library with ctypes, configure return types, create/list/complete goals, memory management (always call aplan_string_free), error handling (aplan_last_error)
- **WASM/npm** — Install @agenticamem/planning, import and instantiate, browser vs Node.js usage
- **MCP tool integration** — Configure as MCP server, tool call patterns, planning_goal operations, planning_decision workflow, planning_progress queries
- **Bridge implementation** — How to implement a custom bridge trait (e.g., MemoryBridge for connecting to agentic-memory), trait method signatures, registration with engine
- **Sister integration** — Connecting with agentic-memory (persist goals as memories), agentic-identity (sign decisions), agentic-time (deadline management), agentic-contract (commitment governance)
- **File persistence** — When to use file-backed mode, save/load patterns, workspace management

### R2: playbooks-agent-integration.md (~120 lines)

Ready-to-use agent integration patterns:

- **30-second quick start** — Minimal MCP config for Claude Code, first tool call
- **System prompt templates:**
  - **Minimal** (~5 lines) — Basic goal tracking: "Use planning_goal to track goals..."
  - **Standard** (~15 lines) — Goals + decisions + progress: "Track goals with physics, crystallize decisions, monitor momentum..."
  - **Full** (~25 lines) — Complete planning suite: all tools with intention singularity, dream forecasting, federation
- **Agent workflow: Solo planner** — Goal lifecycle from creation through completion with decision points
- **Agent workflow: Decision journal** — Using decision crystallization to maintain decision history with shadow paths
- **Agent workflow: Commitment tracker** — Managing promises with entanglement and stakeholder visibility
- **Multi-agent: Federation** — Setting up federation, member sync, collective dreaming, consensus protocol
- **Tips** — When to use physics vs feelings, when to dream, when to collapse singularity

### R3: runtime-install-sync.md (~80 lines)

Runtime operations:

- **Runtime detection** — How the CLI detects engine mode and state file location
- **Auto-install** — curl installer behavior, binary placement, PATH setup
- **MCP server lifecycle** — Starting, stopping, stdio protocol, Content-Length framing
- **Daemon mode** — Background monitoring with `daemon start`, interval configuration, log access, `daemon stop`
- **Workspace sync** — Creating workspaces, switching contexts, comparing and merging planning state
- **Multi-instance** — How multiple CLI/MCP instances can share a file-backed engine (locking, dirty flags)
- **State migration** — Moving planning state between machines via .aplan files

## Acceptance Criteria

- [ ] integration-guide.md has working code examples for Rust, Python, and MCP
- [ ] playbooks-agent-integration.md has copy-paste system prompt templates at 3 tiers
- [ ] runtime-install-sync.md covers daemon, workspace, and multi-instance patterns
- [ ] All examples use actual tool names and parameter formats from the implementation
