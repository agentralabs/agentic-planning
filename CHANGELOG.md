# Changelog

All notable changes to AgenticPlanning will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Full public documentation suite (23 pages covering API, CLI, FFI, MCP, architecture, guides, troubleshooting, benchmarks, FAQ)
- Sister bridge mesh with NoOp defaults for all 9 sister integrations (Time, Contract, Memory, Identity, Cognition, Vision, Codebase, Comm)
- BridgeConfig for runtime bridge selection and initialization
- Canonical sister kit in `docs/public/ecosystem/`

### Changed
- MCP server now exposes 13 consolidated tools (was 12)
- FFI surface expanded to 30 exported functions (was 4)
- All CLI subcommands implemented (no remaining stubs)

## [0.1.0] - 2026-03-01

### Added
- Core planning engine with living goals, decision crystallization, commitment physics, and progress analytics
- Goal physics model: momentum, gravity, inertia, energy — computed from progress history
- Goal feelings: urgency, neglect, confidence, alignment, vitality — computed on access
- Decision lifecycle with shadow paths, causal chains, and prophecy
- Commitment tracking with stakeholder weights, entanglement types, and breaking cost analysis
- Dream system for scenario exploration and insight generation
- Intention singularity: unified field view with themes, tensions, golden path
- Federation model for multi-agent shared planning
- Soul archives for goal reincarnation with karma
- `.aplan` persistence format with BLAKE3 checksums, atomic writes, crash recovery
- 24 precomputed indexes for O(1) lookups on status, priority, deadline, relationships
- 13 consolidated MCP tools in `agentic-planning-mcp` (stdio and HTTP transport)
- `aplan` CLI with goal, decision, commitment, progress, singularity, dream, federation subcommands
- 30 FFI functions in `agentic-planning-ffi` with AplanResult error codes
- WASM/npm package `@agenticamem/planning` for browser and Node.js
- Python SDK via FFI with ctypes bindings
- Invention system: batch operations, parallel singularity, goal metamorphosis, temporal clones, dream weaving
- Scenario, edge-case, and stress tests for core workflows
- CI guardrails: canonical consistency, command surface, MCP consolidation checks
