---
status: stable
---

# File Format

The `.aplan` file is a JSON-encoded binary format with structured header, entity sections, and footer. It stores the complete planning state in a single portable file.

## Overview

```
┌─────────────────────────┐
│     AplanHeader         │  Entity counts, section offsets, BLAKE3 checksum
├─────────────────────────┤
│     Goals section       │  HashMap<GoalId, Goal>
│     Decisions section   │  HashMap<DecisionId, Decision>
│     Commitments section │  HashMap<CommitmentId, Commitment>
│     Dreams section      │  HashMap<DreamId, Dream>
│     Federations section │  HashMap<FederationId, Federation>
│     Soul archive        │  HashMap<GoalId, GoalSoulArchive>
│     Indexes             │  PlanIndexes (24 structures)
├─────────────────────────┤
│     AplanFooter         │  File size, write count, integrity marker
└─────────────────────────┘
```

## Magic Number and Version

Every `.aplan` file starts with the magic bytes `PLAN` (0x504C414E) followed by a version field.

- **Magic:** `[0x50, 0x4C, 0x41, 0x4E]` — identifies the file as an AgenticPlanning artifact
- **Version:** `u16` — currently `1`. Used for forward compatibility; older files are loaded correctly by newer engines

## Header Structure

The `AplanHeader` is a 128-byte C-repr packed struct:

| Field | Type | Description |
|-------|------|-------------|
| `magic` | `[u8; 4]` | File identifier: `PLAN` |
| `version` | `u16` | Format version (currently 1) |
| `flags` | `u32` | Reserved for future feature flags |
| `created_at` | `i64` | Creation timestamp (nanoseconds since epoch) |
| `modified_at` | `i64` | Last modification timestamp |
| `goal_count` | `u32` | Number of goals in the file |
| `decision_count` | `u32` | Number of decisions |
| `commitment_count` | `u32` | Number of commitments |
| `dream_count` | `u32` | Number of dreams |
| `federation_count` | `u32` | Number of federations |
| `goal_section_offset` | `u64` | Byte offset to goals section |
| `decision_section_offset` | `u64` | Byte offset to decisions section |
| `commitment_section_offset` | `u64` | Byte offset to commitments section |
| `dream_section_offset` | `u64` | Byte offset to dreams section |
| `federation_section_offset` | `u64` | Byte offset to federations section |
| `index_section_offset` | `u64` | Byte offset to index section |
| `checksum` | `[u8; 32]` | BLAKE3 hash of all entity data |
| `reserved` | `[u8; 14]` | Reserved for future use |

## Entity Sections

Each section contains a JSON-serialized HashMap of entities. The internal format uses `serde_json` serialization with `HashMap<EntityId, Entity>` structures:

- **Goals** — Full goal state including physics, feelings, progress history, blockers, relationships, tags, soul
- **Decisions** — Paths (chosen and shadow), reasoning, causal chains, crystallization timestamp
- **Commitments** — Promise, stakeholder, entanglements, breaking cost, fulfillment state
- **Dreams** — Scenarios, obstacles, insights, sub-goal seeds, timestamps
- **Federations** — Members, shared goals, sync status, consensus records
- **Soul archive** — Reincarnation history and karma for abandoned-then-reborn goals

## Footer Structure

The `AplanFooter` is a 64-byte C-repr packed struct:

| Field | Type | Description |
|-------|------|-------------|
| `file_size` | `u64` | Size of serialized entity data |
| `write_count` | `u64` | Monotonic counter incremented on each save |
| `last_session` | `[u8; 16]` | UUID bytes of the last writing session |
| `integrity` | `[u8; 8]` | Integrity marker: `PLANEND\0` (0x504C414E454E4400) |
| `footer_checksum` | `[u8; 16]` | BLAKE3 hash of footer payload (truncated to 16 bytes) |
| `reserved` | `[u8; 8]` | Reserved for future use |

## Checksums

BLAKE3 is used for all integrity verification:

**Header checksum:** Computed over all entity data using deterministic ordering. HashMap keys are collected into BTreeMaps before serialization to ensure the same data always produces the same hash:

```
checksum = blake3(serialize(
    sorted_goals, sorted_decisions, sorted_commitments,
    sorted_dreams, sorted_federations, sorted_souls, indexes
))
```

**Footer checksum:** Computed over `(file_size, write_count, last_session)` and truncated to 16 bytes.

**Legacy support:** Files with an all-zero checksum field are treated as legacy (pre-checksum) files and loaded without verification.

## Atomic Writes

Saves use a write-then-rename strategy to prevent corruption:

1. Serialize the full `PersistedPlan` struct (header + all entities + indexes + footer)
2. Write to `<path>.aplan.tmp`
3. Call `fsync` to flush to disk
4. Rename `.aplan.tmp` to `.aplan` (atomic on POSIX)

If the process crashes between steps 2 and 4, only the `.tmp` file exists. The `.aplan` file is either the previous valid version or absent.

## Crash Recovery

On `PlanningEngine::open()`:

| `.aplan` exists | `.aplan.tmp` exists | Action |
|:---:|:---:|--------|
| No | No | Create new empty file |
| No | Yes | Recover: rename `.tmp` to `.aplan`, then load |
| Yes | No | Normal load |
| Yes | Yes | Load `.aplan` (completed write), delete stale `.tmp` |

## Verification on Load

When loading an `.aplan` file:

1. Deserialize the JSON into a `PersistedPlan` struct
2. Verify `magic == PLAN` and `version == 1`
3. Verify `integrity == PLANEND\0`
4. If checksum is non-zero: recompute BLAKE3 over entity data using BTreeMap ordering and compare
5. If checksum mismatches: return `CorruptedFile` error
6. Populate engine stores and indexes from the loaded data

## Size Characteristics

Typical `.aplan` file sizes (JSON-encoded, pretty-printed):

| Project Scale | Goals | Decisions | Commitments | Approximate Size |
|---------------|-------|-----------|-------------|------------------|
| Small | 5 | 3 | 2 | ~15 KB |
| Medium | 20 | 15 | 10 | ~60 KB |
| Large | 50 | 40 | 30 | ~150 KB |
| Very large | 100+ | 80+ | 60+ | ~300+ KB |

Growth is roughly linear: each goal adds ~2 KB, each decision ~1.5 KB, each commitment ~1 KB. Index overhead is proportional to entity count.

## Portability

The `.aplan` file is self-contained. Copy it to another machine and the full planning state travels with it — goals, decisions, commitments, dreams, federations, soul archives, and all indexes. No external database or service required.

The format is version-safe across minor releases. The `version` field in the header ensures older files can be loaded by newer engines with appropriate migration logic.
