# Agentic-Planning Comprehensive Gap Report

**Date:** 2026-03-01
**Scope:** All 6 crates in the agentic-planning workspace
**Total source:** ~10,858 lines across all crates

---

## Executive Summary

The agentic-planning codebase has a rich domain model (types.rs) and functional
scaffolding, but roughly 40% of the implementation consists of stubs, hardcoded
values, or placeholder logic. The 8 public documentation files are all empty
placeholders. The FFI layer covers only 4 of ~25 needed functions.

**By severity:**

| Severity | Count | Description |
|----------|-------|-------------|
| Critical | 12 | Server-crashing unwrap() calls, data-loss file format bugs |
| High     | 18 | Stub functions returning fake data, ignored parameters |
| Medium   | 15 | Hardcoded constants, missing validations |
| Low      | 10 | Dead code, unused fields, empty docs |

---

## 1. Core Engine (`agentic-planning/src/`)

### 1.1 Stub Functions Returning Fake Data

| File | Line | Function | Issue |
|------|------|----------|-------|
| query_engine.rs | 170 | `build_chain_recursive()` | Always assigns `CausalityType::Enables` regardless of actual relationship |
| query_engine.rs | 221 | `decision_archaeology()` | Sets `was_reasonable: true` for ALL decisions unconditionally |
| query_engine.rs | 231 | `decision_archaeology()` | `insights` always `Vec::new()`, never generates actual insights |
| query_engine.rs | 322-323 | `scan_blocker_prophecy()` | Hardcoded `prediction_confidence: 0.7`, `days_until_materialization: 7.0` |
| query_engine.rs | 628-647 | `generate_projected_timeline()` | Returns 2 hardcoded template events, ignores actual goal state |
| query_engine.rs | 649-651 | `project_final_state()` | Returns format string, no real projection |
| query_engine.rs | 657-663 | `project_path_timeline()` | Returns 1 hardcoded event |
| query_engine.rs | 666-668 | `project_path_final_state()` | Returns format string |
| query_engine.rs | 479-492 | `synthesize_vision()` | Just joins top 3 goal titles, no actual synthesis |
| query_engine.rs | 506-524 | `find_tensions()` | Only checks urgency divergence > 0.4, ignores all other tension types |
| lib.rs | 168-173 | `check_success_criteria()` | Marks ALL criteria as achieved unconditionally |
| lib.rs | 282-284 | `calculate_chain_bonus()` | Returns hardcoded 0.1, ignores commitment id |

### 1.2 Ignored Parameters

| File | Line | Function | Ignored Parameter |
|------|------|----------|-------------------|
| write_engine.rs | 117 | `pause_goal()` | `_reason: &str` — reason is never stored |
| write_engine.rs | 752 | `break_commitment()` | `_reason: &str` — reason is never stored |
| write_engine.rs | 798 | `renegotiate_commitment()` | Always sets `accepted: true` without negotiation logic |

### 1.3 Hardcoded Magic Numbers

| File | Line | Value | Context |
|------|------|-------|---------|
| write_engine.rs | 733 | `0.5` | trust_gained multiplier in fulfill_commitment |
| write_engine.rs | 184-185 | `0.2`, `0.1` | energy/momentum boost in release_completion_energy |
| write_engine.rs | 739 | `0.3` | entanglement energy propagation factor |
| write_engine.rs | 799 | `-0.05` | renegotiation trust_impact |
| query_engine.rs | 322 | `0.7` | blocker prediction confidence |
| query_engine.rs | 323 | `7.0` | days_until_materialization |
| lib.rs | multiple | `86_400.0 * 1e9` | time constants (nanoseconds in a day) |

### 1.4 Inventions (inventions.rs) — ALL Serial Wrappers

All 4 "parallel" functions just call their serial counterparts:

- `calculate_singularity_parallel()` → calls `get_intention_singularity()`
- `scan_blockers_parallel()` → calls `scan_blocker_prophecy()`
- `progress_echoes_parallel()` → calls `listen_progress_echoes()`
- `create_goals_batch()` → serial loop over `create_goal()`

### 1.5 Indexes Never Populated

| File | Line | Field | Issue |
|------|------|-------|-------|
| indexes.rs | 29 | `goal_relationships` | HashMap defined, never written to anywhere |
| indexes.rs | 30 | `commitment_entanglements` | HashMap defined, never written to anywhere |
| indexes.rs | rebuild() | — | Only processes goals, decisions, commitments; ignores dreams and federations |

### 1.6 File Format Integrity Gaps (file_format.rs)

| Line | Issue | Impact |
|------|-------|--------|
| 122 | `flags: 0` always | Feature flags never set |
| 141 | `write_count: 1` always | Never incremented, write history lost |
| 142 | `last_session: [0; 16]` | Session tracking broken |
| 143 | `footer_checksum: [0; 16]` | Checksum never computed — silent corruption |
| 130-135 | All section offsets = 0 | Multi-section format not implemented |
| load() | No checksum verification | Corrupted files loaded silently |
| — | No .tmp crash recovery | Orphaned temp files on crash |

### 1.7 Validation Gaps (validation.rs)

Missing checks:
- `emotional_weight` bounds (should be 0.0..=1.0)
- Self-dependency detection (goal depending on itself)
- Stakeholder importance bounds
- Deadline reachability (deadline in past)
- Dead parent detection (child of deleted goal)
- `_keep_commitment_status` dead code at line 237

---

## 2. MCP Server (`agentic-planning-mcp/src/server.rs`)

### 2.1 Critical: 28 unwrap() Calls

Every `get_string_param()` / `get_number_param()` call uses `.unwrap()`, which
panics the MCP server process on any missing or malformed parameter. Sister
servers (memory, vision) use `ok_or_else()` with proper MCP error responses.

### 2.2 Missing Endpoints

- `prompts/get` — NOT IMPLEMENTED (returns empty)
- Missing resources: `planning://consensus/{id}`, `planning://workspace/{id}`, `planning://dreams/{id}`

### 2.3 No Input Validation

Parameters extracted from JSON are used directly without bounds checking,
length limits, or sanitization.

---

## 3. CLI (`agentic-planning-cli/src/main.rs`)

### 3.1 Stub Commands (8 total)

| Subcommand | Issue |
|------------|-------|
| `progress blockers` | Stub — not implemented |
| `progress echoes` | Stub — not implemented |
| `singularity collapse` | Stub — not implemented |
| `singularity path` | Stub — not implemented |
| `singularity tensions` | Stub — not implemented |
| `singularity themes` | Stub — not implemented |
| `singularity vision` | Stub — not implemented |
| `federation consensus` | Stub — not implemented |

### 3.2 Output Format Broken

`OutputFormat` enum supports `Table`, `Text`, `Json` but `emit_output()` ALWAYS
serializes to JSON regardless of selected format.

### 3.3 Hardcoded Values

- Stakeholder role: always `"stakeholder"`
- Stakeholder importance: always `0.6`
- Entanglement strength: always `0.8`

### 3.4 Serve Mode Gaps

- HTTP mode not implemented (only stdio)
- Stdio serve loop doesn't persist engine mutations to disk

---

## 4. FFI (`agentic-planning-ffi/src/lib.rs`)

### 4.1 Severely Incomplete

Only 4 functions exist:
1. `aplan_engine_new()` — create engine
2. `aplan_engine_free()` — destroy engine
3. `aplan_goal_create()` — create a goal
4. `aplan_string_free()` — free string

Compared to agentic-memory FFI which has 30+ functions. Missing:
- All decision operations
- All commitment operations
- All dream operations
- All federation operations
- All query/search operations
- Error code enum
- Version/capability query
- File load/save

### 4.2 No Error Codes

No `AplanError` enum. Errors return null pointers with no way to inspect what
went wrong.

### 4.3 No Tests

Zero FFI tests.

---

## 5. Bridges (`agentic-planning-bridges/src/lib.rs`)

### 5.1 BridgeConfig Unused

`BridgeConfig` struct defined but never consumed by any bridge constructor.

### 5.2 NoOp Bridges Complete (by design)

All 7 Null* bridges are correctly implemented as no-ops. This is the expected
pattern — real bridges are used when integrated with other sisters.

---

## 6. Documentation (docs/public/)

### 6.1 ALL 8 Core Docs Are Empty Placeholders

Each file contains only the title and nothing else:
- mcp-tools.md, mcp-resources.md, mcp-prompts.md
- cli-reference.md, command-surface.md
- api-reference.md, ffi-reference.md, concepts.md

---

## 7. Tests

### 7.1 Current Coverage

~49 tests across all crates. Only happy-path scenarios covered.

### 7.2 Missing Test Categories

- Error path tests (invalid inputs, missing entities)
- Intermediate state transition tests (goal lifecycle, decision crystallization)
- File format round-trip corruption tests
- MCP parameter validation tests
- CLI output format tests
- FFI tests (zero exist)
- Bridge failure mode tests
- Concurrent access tests

---

## Implementation Priority

1. **P0 — Server stability:** Fix 28 unwrap() panics in MCP server
2. **P0 — Data integrity:** Fix file_format checksum, write_count, crash recovery
3. **P1 — Core stubs:** Implement real logic for 12 stub functions
4. **P1 — Indexes:** Populate goal_relationships and commitment_entanglements
5. **P1 — Validation:** Add missing bounds checks and structural validation
6. **P2 — CLI completeness:** Implement 8 stub commands + output formatting
7. **P2 — FFI expansion:** Add 20+ FFI functions with error codes
8. **P3 — Documentation:** Fill all 8 doc files
9. **P3 — Tests:** Add error path and edge case tests
