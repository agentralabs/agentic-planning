# SPEC-13: Reference & Operations Documentation

**Priority:** P2
**Files:** `docs/public/troubleshooting.md`, `docs/public/benchmarks.md`, `docs/public/faq.md`
**Estimated changes:** ~260 lines total

## Problem

Users hitting errors have no troubleshooting guide. There are no performance benchmarks to set expectations. Common questions have no answers. These three docs are the operational support layer — without them, users are on their own when things go wrong.

## Requirements

### R1: troubleshooting.md (~100 lines)

Common issues with diagnosis and resolution:

- **MCP connection issues**
  - Server won't start: check binary path, permissions, stdio mode
  - Tool calls fail: verify JSON-RPC format, check parameter names/types
  - Server crashes: likely invalid parameter (unwrap panic) — check aplan_last_error equivalent
- **File format issues**
  - Corrupted .aplan file: checksum mismatch on load, recovery from .aplan.tmp backup
  - File won't load: version mismatch, missing magic number, truncated write
  - Permission errors: file/directory permissions, disk full
- **CLI errors**
  - Unknown command: check version, command spelling
  - Entity not found: verify UUID format, check if goal/decision/commitment exists
  - Invalid transition: check current entity status (can't complete a Draft goal, can't crystallize a Crystallized decision)
- **Engine state issues**
  - Goals stuck in wrong state: lifecycle transition rules, valid state diagram
  - Indexes out of sync: trigger rebuild with engine reload
  - Stale feelings/physics: recalculated on access, but based on last activity time
- **Performance issues**
  - Slow singularity collapse: scales with goal count, use indexes
  - Large file sizes: entity count growth, consider archiving completed goals
  - Memory usage: in-memory engine holds all entities, file-backed mode for large datasets
- **Error code reference** — AplanResult enum (Ok=0, NullPointer=1, InvalidUtf8=2, EngineError=3, NotFound=4, ValidationError=5, IoError=6, SerializationError=7)

### R2: benchmarks.md (~80 lines)

Performance characteristics:

- **Operation latency**
  - Goal CRUD: <1ms (create, get, update, delete)
  - Decision crystallization: <1ms
  - Commitment operations: <1ms
  - Singularity collapse: ~5-15ms (depends on goal count)
  - Dream generation: ~2-5ms
  - Blocker scan: ~1-3ms
  - File save: ~5-20ms (depends on entity count)
  - File load: ~5-30ms (includes index rebuild)
- **Memory footprint**
  - Per goal: ~2-4KB in memory
  - Per decision: ~1-3KB
  - Per commitment: ~1-2KB
  - 100-goal project: ~500KB total
  - 1000-goal project: ~5MB total
- **File size scaling**
  - Empty engine: ~1KB
  - 10 goals: ~25KB
  - 100 goals with decisions/commitments: ~300KB
  - 1000 goals: ~3MB
- **Batch operations**
  - create_goals_batch: ~20% faster than serial for 50+ goals (single index rebuild)
  - calculate_singularity_parallel: ~15% faster for 100+ goals (pre-filtering)
- **Hardware assumptions** — Benchmarks measured on typical development machine (M-series Mac, 16GB RAM), single-threaded (engine is !Sync)
- **Scaling guidance** — Linear scaling up to ~10,000 entities, index-assisted queries remain O(1) for lookups

### R3: faq.md (~80 lines)

Frequently asked questions:

- **What is AgenticPlanning?** — A persistent planning engine that gives AI agents living goals, crystallized decisions, and tracked commitments
- **What is an .aplan file?** — A portable binary file containing all planning state (goals, decisions, commitments, dreams, federations) with checksums and crash recovery
- **How is this different from a TODO list?** — Goals have physics (momentum, gravity), feelings (urgency, confidence), and lifecycle states. Decisions preserve shadow paths. Commitments model entanglement. A TODO list is a flat checklist.
- **Can multiple agents share planning state?** — Yes, via federation model. Agents join a federation around a root goal, each owns sub-goals, and they sync state through the coordinator.
- **Does it work with Claude Code?** — Yes, configure the MCP server in settings.json. The 13 MCP tools give Claude full planning capabilities.
- **How big do .aplan files get?** — ~300KB for a 100-goal project. ~3MB for 1000 goals. Files use binary serialization with section offsets.
- **Is it production-ready?** — The core engine passes 80 tests including stress scenarios. The MCP server, CLI, and FFI are feature-complete. Bridge implementations are no-op by default (real integrations come from sister packages).
- **What happens if the process crashes mid-write?** — Atomic writes: data goes to .aplan.tmp first, then rename. If crashed, recovery detects the temp file on next load.
- **Can I use it from Python/JavaScript/C?** — Yes. Python via FFI (ctypes), JavaScript via WASM (@agenticamem/planning npm package), C via the 27 exported FFI functions.
- **What are "goal feelings"?** — Computed health indicators: urgency (deadline pressure), neglect (time since attention), confidence (success likelihood), alignment (singularity fit), vitality (overall health).
- **What is the "intention singularity"?** — The convergence point where all goals, decisions, and commitments align into a unified vision with a golden path (optimal execution sequence).
- **How do I back up planning state?** — Copy the .aplan file. It's a self-contained binary with checksums. Version-safe across minor releases.

## Acceptance Criteria

- [ ] troubleshooting.md covers MCP, CLI, file format, and engine state issues
- [ ] benchmarks.md has concrete numbers (ms, KB, MB) matching actual performance
- [ ] faq.md answers the 12 most common questions with practical answers
- [ ] Error code table matches AplanResult enum from FFI
- [ ] All troubleshooting steps are actionable (not "contact support")
