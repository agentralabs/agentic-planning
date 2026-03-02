# SPEC-09: Getting Started Documentation

**Priority:** P1
**Files:** `docs/public/overview.md`, `docs/public/quickstart.md`, `docs/public/installation.md`
**Estimated changes:** ~280 lines total

## Problem

Users encountering AgenticPlanning have no way to understand what it is, why they should use it, or how to start. The overview is empty, the quickstart has bare commands with no context, and installation lists methods without explaining when to use each.

## Requirements

### R1: overview.md (~120 lines)

Document what AgenticPlanning is and why it exists:

- **Opening paragraph** — One-sentence definition: what AgenticPlanning does (persistent planning engine for AI agents)
- **Problem statement** — Goal drift across sessions, decision amnesia, commitment overload, progress blindness
- **Core capabilities** — Living goals with physics/feelings, decision crystallization with shadow paths, commitment entanglement, dream forecasting, intention singularity
- **The .aplan artifact** — Portable binary file containing all planning state, analogous to .amem for memory
- **Five surfaces** — MCP server, CLI, Rust API, C FFI, WASM/npm (one engine, many interfaces)
- **Sister ecosystem** — How planning relates to memory, vision, identity, codebase, time, contract, comm
- **Who should use this** — AI agent builders, multi-agent coordinators, long-running workflow systems
- **Quick example** — 5-line CLI session showing goal create → progress → feelings

### R2: quickstart.md (~80 lines)

Step-by-step first experience:

- **Prerequisites** — Rust toolchain or pre-built binary
- **Step 1: Install** — `cargo install agentic-planning-cli` with verification
- **Step 2: Create a goal** — Full command with --intention and --priority flags, explain the output
- **Step 3: Track progress** — `goal progress <ID> 30 --note "Started implementation"`, explain physics changes
- **Step 4: Check feelings** — `goal feelings <ID>`, explain urgency/confidence/vitality
- **Step 5: Make a decision** — `decision create`, add options, crystallize
- **Step 6: Start MCP server** — `aplan serve` for Claude Code integration
- **Next steps** — Links to CLI reference, MCP tools, concepts, integration guide

### R3: installation.md (~80 lines)

Complete installation guide:

- **Pre-built binary** — curl installer with platform detection, verification
- **Cargo install** — `cargo install agentic-planning-cli`, requires Rust 1.75+
- **npm package** — `npm install @agenticamem/planning` for WASM/Node.js usage
- **pip package** — `pip install agentic-planning` for Python FFI bindings
- **From source** — Clone, `cargo build --release`, binary locations
- **MCP server setup** — How to configure in Claude Code settings.json, stdio mode, example config block
- **Verification** — `agentic-planning version`, `agentic-planning status`
- **Upgrading** — How to upgrade each method, state file compatibility

## Acceptance Criteria

- [ ] overview.md explains what, why, and how in < 2 minutes of reading
- [ ] quickstart.md gets a user from zero to working in under 10 minutes
- [ ] installation.md covers all 4 install methods with MCP setup
- [ ] All code examples use real commands with realistic output
- [ ] Cross-references link to relevant deeper docs
