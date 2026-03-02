---
status: stable
---

# Benchmarks

Performance characteristics of AgenticPlanning measured on a typical development machine (Apple Silicon, 16 GB RAM, single-threaded).

## Operation Latency

| Operation | Typical Latency | Notes |
|-----------|----------------|-------|
| Goal create | <1 ms | Includes index update |
| Goal get | <1 ms | Direct HashMap lookup |
| Goal update (status change) | <1 ms | Index update is incremental |
| Decision create | <1 ms | |
| Decision crystallize | <1 ms | Shadow path serialization included |
| Commitment create | <1 ms | |
| Commitment fulfill/break | <1 ms | |
| Progress record | <1 ms | Includes momentum recalculation |
| Blocker scan | 1-3 ms | Scales with active goal count |
| Dream generation | 2-5 ms | Depends on goal complexity |
| Singularity collapse | 5-15 ms | O(n) over active goals |
| File save | 5-20 ms | BTreeMap sorting + BLAKE3 + fsync |
| File load | 5-30 ms | Includes full index rebuild |

All CRUD operations (create, read, update, delete) on individual entities complete in sub-millisecond time. The engine's HashMap-based stores provide O(1) lookups.

## Memory Footprint

| Component | Size | Notes |
|-----------|------|-------|
| Per goal | 2-4 KB | Includes physics, feelings, progress history, tags, relationships |
| Per decision | 1-3 KB | Includes shadow paths, causal chain references |
| Per commitment | 1-2 KB | Includes entanglement references |
| Per dream | 1-2 KB | Scenarios, insights, seeds |
| PlanIndexes (base) | ~10 KB | 24 index structures, empty |
| PlanIndexes (100 goals) | ~50 KB | Grows with entity count |

**Project-scale estimates:**

| Scale | Goals | Decisions | Commitments | Memory |
|-------|-------|-----------|-------------|--------|
| Small | 10 | 5 | 5 | ~80 KB |
| Medium | 50 | 30 | 20 | ~350 KB |
| Large | 100 | 60 | 40 | ~500 KB |
| Very large | 500 | 200 | 100 | ~2.5 MB |

## File Size Scaling

| Content | Approximate Size |
|---------|-----------------|
| Empty engine | ~1 KB |
| 5 goals | ~15 KB |
| 10 goals, 5 decisions | ~25 KB |
| 20 goals, 15 decisions, 10 commitments | ~60 KB |
| 50 goals, 40 decisions, 30 commitments | ~150 KB |
| 100 goals, 80 decisions, 60 commitments | ~300 KB |

Growth is roughly linear:
- Each goal adds ~2 KB to the file
- Each decision adds ~1.5 KB
- Each commitment adds ~1 KB
- Index overhead grows proportionally to entity count

## Index Performance

Index-assisted queries provide fast lookups regardless of total entity count:

| Query | Complexity | Index Used |
|-------|-----------|------------|
| Goals by status | O(1) | `goals_by_status` |
| Goals by priority | O(1) | `goals_by_priority` |
| Goals by deadline (range) | O(log n) | `goals_by_deadline` (BTreeMap) |
| Root goals | O(1) | `root_goals` |
| Active/blocked goals | O(1) | `active_goals` / `blocked_goals` |
| Decisions by goal | O(1) | `decisions_by_goal` |
| Commitments due soon | O(log n) | `commitments_by_due` (BTreeMap) |
| Urgent items | O(1) | `urgent_items` |

## Scaling Guidance

- **Sweet spot:** 5-100 goals. All operations are sub-millisecond except singularity collapse.
- **Comfortable range:** Up to 1,000 entities total. File sizes stay under 3 MB. Load times under 100 ms.
- **Upper bound:** ~10,000 entities. Linear scaling holds. File operations become the bottleneck (~200+ ms saves).
- **Not designed for:** Millions of entities. Use a database-backed planning system for that scale.

The engine is single-threaded (`PlanningEngine` is `!Sync`). All benchmarks reflect sequential operation. MCP server handles one tool call at a time.

## Checksum Overhead

BLAKE3 hashing adds minimal overhead to save operations:

| Entity Count | Checksum Time | % of Total Save |
|-------------|---------------|-----------------|
| 10 | <1 ms | ~5% |
| 100 | ~1 ms | ~10% |
| 1000 | ~3 ms | ~15% |

BTreeMap sorting (for deterministic checksums) is the larger contributor to save latency at scale.
