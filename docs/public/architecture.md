---
status: stable
---

# Architecture

AgenticPlanning is a Cargo workspace with five crates, each serving a distinct surface.

## Workspace Layout

```
agentic-planning/
  crates/
    agentic-planning/          Core engine, types, indexes, file format, inventions
    agentic-planning-mcp/      MCP server (JSON-RPC 2.0 over stdio)
    agentic-planning-cli/      CLI binary (`aplan`) with all commands
    agentic-planning-ffi/      C-compatible FFI (27 exported functions)
    agentic-planning-bridges/  Sister integration traits (9 bridge traits)
  npm/                         WASM build for Node.js and browser
```

## Core Engine

The `PlanningEngine` struct is the central type. Everything flows through it.

```rust
pub struct PlanningEngine {
    path: Option<PathBuf>,           // None = in-memory, Some = file-backed
    dirty: bool,                     // Tracks unsaved mutations
    goal_store: HashMap<GoalId, Goal>,
    decision_store: HashMap<DecisionId, Decision>,
    commitment_store: HashMap<CommitmentId, Commitment>,
    dream_store: HashMap<DreamId, Dream>,
    federation_store: HashMap<FederationId, Federation>,
    soul_archive: HashMap<GoalId, GoalSoulArchive>,
    indexes: PlanIndexes,            // 24 precomputed index structures
    write_count: u64,                // Monotonic counter for file versioning
    session_id: Uuid,                // Unique per engine instance
}
```

**Two modes:**

- `PlanningEngine::in_memory()` — No persistence. State lives only in memory. Useful for tests, ephemeral sessions, and WASM.
- `PlanningEngine::open(path)` — File-backed. Loads existing `.aplan` file or creates a new one. Saves on mutation.

## Five-Entity Model

The engine manages five entity types, each with its own store and lifecycle:

| Entity | Key Type | Purpose |
|--------|----------|---------|
| **Goal** | `GoalId` | Intentions with physics, feelings, lifecycle, relationships |
| **Decision** | `DecisionId` | Choices with shadow paths, causal chains, crystallization |
| **Commitment** | `CommitmentId` | Promises with stakeholders, entanglements, breaking costs |
| **Dream** | `DreamId` | Forward-looking simulations tied to goals |
| **Federation** | `FederationId` | Multi-agent coordination with shared goals and sync state |

Supporting types: `GoalSoulArchive` (karma and reincarnation history), `PlanIndexes` (query acceleration).

## Data Flow

A request flows through four layers:

```
Client (MCP/CLI/FFI/Rust API)
  → Tool dispatch (server.rs / main.rs / FFI shim)
    → Write engine (write_engine.rs) or Query engine (query_engine.rs)
      → Validation (validation.rs)
      → Store mutation (HashMap insert/update)
      → Index update (indexes.rs — incremental)
      → Optional file save (file_format.rs — atomic write)
```

**Write path:** Validates input → mutates store → updates indexes incrementally → marks dirty → saves if file-backed.

**Query path:** Reads from indexes for fast lookups (goals by status, commitments by deadline) or from stores directly for single-entity fetches.

## Index System

`PlanIndexes` maintains 24 precomputed structures for fast queries:

**Goal indexes:**
- `goals_by_status` — `HashMap<GoalStatus, Vec<GoalId>>`
- `goals_by_priority` — `HashMap<Priority, Vec<GoalId>>`
- `goals_by_deadline` — `BTreeMap<Timestamp, Vec<GoalId>>` (sorted)
- `goals_by_parent` — `HashMap<GoalId, Vec<GoalId>>`
- `goals_by_tag` — `HashMap<String, Vec<GoalId>>`
- `root_goals`, `active_goals`, `blocked_goals` — `Vec<GoalId>`
- `goal_relationships` — `HashMap<GoalId, Vec<(GoalId, GoalRelationship)>>`

**Decision indexes:**
- `decisions_by_goal`, `decisions_by_time`, `pending_decisions`, `regretted_decisions`, `decision_chains`

**Commitment indexes:**
- `commitments_by_due` (BTreeMap, sorted), `commitments_by_stakeholder`, `commitments_by_goal`, `active_commitments`, `at_risk_commitments`, `commitment_entanglements`

**Cross-entity:**
- `urgent_items` — Goals and commitments due within 7 days
- `dreams_by_goal`, `federations_by_member`, `federations_by_sync`

Indexes are updated incrementally on each mutation. Full rebuild is available via `rebuild_full()` and runs automatically on file load.

## Bridge Architecture

Nine trait-based bridges connect planning to the sister ecosystem:

| Bridge | Sister | Purpose |
|--------|--------|---------|
| `TimeBridge` | agentic-time | Deadline context, scheduling, decay |
| `ContractBridge` | agentic-contract | Policy checks, compliance, approval |
| `MemoryBridge` | agentic-memory | Persist goals/decisions as memories |
| `IdentityBridge` | agentic-identity | Sign decisions, verify agents |
| `CognitionBridge` | — | User model, preference prediction |
| `VisionBridge` | agentic-vision | Evidence capture and linking |
| `CodebaseBridge` | agentic-codebase | Code symbol linking, impact analysis |
| `CommBridge` | agentic-comm | Multi-agent messaging |
| `NoOp*` | — | Default implementations (return neutral values) |

Each bridge trait defines 3-5 methods. In standalone mode, `NoOp` implementations keep the engine self-contained with no external dependencies. Real implementations connect to sister MCP servers or direct library calls.

## Persistence Layer

File-backed engines use the `.aplan` binary format:

- **Atomic writes** — Write to `.aplan.tmp`, fsync, rename to `.aplan`
- **BLAKE3 checksums** — Over all entity data, verified on load
- **Crash recovery** — If only `.aplan.tmp` exists, rename it to `.aplan`
- **Deterministic serialization** — BTreeMap ordering ensures consistent checksums

See [File Format](file-format.md) for the full specification.

## Crate Dependencies

```
agentic-planning-cli
  ├── agentic-planning (core)
  ├── agentic-planning-mcp
  └── agentic-planning-bridges

agentic-planning-mcp
  └── agentic-planning (core)

agentic-planning-ffi
  └── agentic-planning (core)

agentic-planning-bridges
  └── agentic-planning (core)
```

The core crate has no dependency on any sister package. Bridges are the only integration point, and they're trait-based — compile-time zero-cost when using NoOp defaults.
