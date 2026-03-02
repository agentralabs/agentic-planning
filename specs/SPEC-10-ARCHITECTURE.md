# SPEC-10: Architecture & Internals Documentation

**Priority:** P1
**Files:** `docs/public/architecture.md`, `docs/public/file-format.md`, `docs/public/configuration.md`
**Estimated changes:** ~350 lines total

## Problem

Developers and contributors have no documentation about how AgenticPlanning is built. The workspace has 7 crates, a binary format, bridge traits, and multiple configuration surfaces — none of which are documented.

## Requirements

### R1: architecture.md (~150 lines)

Document the system architecture:

- **Workspace layout** — ASCII tree of all 7 crates with one-line purpose:
  - `agentic-planning` — Core engine, types, query/write engines, validation, indexes, file format, inventions
  - `agentic-planning-mcp` — MCP server (JSON-RPC 2.0 over stdio)
  - `agentic-planning-cli` — CLI binary with all commands
  - `agentic-planning-ffi` — C-compatible FFI (27 exported functions)
  - `agentic-planning-bridges` — Sister integration traits (9 bridges)
  - `npm/wasm` — WASM build for Node.js/browser
- **Core engine design** — PlanningEngine struct: stores (goals, decisions, commitments, dreams, federations, soul_archives), indexes (PlanIndexes), session tracking, dirty flag, persistence path
- **Data flow** — How a request flows: MCP JSON-RPC → server.rs tool dispatch → write_engine/query_engine → indexes update → optional file save
- **Five-entity model** — Goal, Decision, Commitment, Dream, Federation with their relationships
- **Bridge architecture** — 9 trait-based bridges (Time, Contract, Memory, Identity, Cognition, Vision, Codebase, Comm + NoOp default), null implementations for standalone mode, real implementations when integrated with sisters
- **Persistence layer** — Binary .aplan format with atomic writes, checksum verification, crash recovery
- **Index system** — 8+ index structures (root_goals, active_goals, blocked_goals, goals_by_deadline, goal_relationships, etc.) rebuilt on load, incrementally updated on mutations

### R2: file-format.md (~120 lines)

Document the .aplan binary format:

- **Overview** — Binary file format with header, sections, footer, checksums
- **Magic number** — `PLAN` (0x504C414E), version byte
- **Header structure** — Entity counts (goals, decisions, commitments, dreams, federations), section offsets, flags, reserved bytes
- **Sections** — JSON-serialized entity stores: goals, decisions, commitments, dreams, federations, soul_archives, indexes
- **Footer structure** — File size, write count, session ID, integrity marker `DONE` (0x444F4E45), checksums
- **Checksums** — BLAKE3 over section data, deterministic ordering via BTreeMap
- **Atomic writes** — Write to `.aplan.tmp`, fsync, rename to `.aplan`
- **Crash recovery** — If `.aplan.tmp` exists without `.aplan`, recovery restores from tmp
- **Verification** — Checksum validation on load, header/footer integrity checks
- **Size characteristics** — Typical sizes per entity count, growth rate

### R3: configuration.md (~80 lines)

Document all configuration options:

- **CLI global options** — `--file <PATH>` (state file), `--format text|json|table`, `--json` (shorthand), `--verbose`
- **Engine modes** — In-memory (default, no persistence) vs file-backed (`PlanningEngine::from_file()`)
- **MCP server** — Stdio mode (default), planned HTTP mode, Content-Length framed JSON-RPC
- **Claude Code integration** — settings.json MCP server configuration block with command and args
- **Daemon settings** — `--interval_secs` for background monitoring, log location
- **Bridge configuration** — BridgeConfig struct, enabling/disabling individual bridges
- **Environment variables** — Any supported env vars for paths, modes, verbosity
- **Serve command** — `--mode stdio|http`, `--port <PORT>` (default 3000)

## Acceptance Criteria

- [ ] architecture.md has ASCII workspace tree matching actual crate structure
- [ ] file-format.md documents every header/footer field from file_format.rs
- [ ] configuration.md covers every configurable surface (CLI, MCP, daemon, bridges)
- [ ] All descriptions match actual implementation (no aspirational docs)
