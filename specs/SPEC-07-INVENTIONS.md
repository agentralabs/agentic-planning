# SPEC-07: Inventions — Real Parallel Operations

**Priority:** P2
**File:** `agentic-planning/src/inventions.rs`
**Estimated changes:** ~100 lines

## Problem

All 4 "parallel" invention functions are serial wrappers that just call the
synchronous version. The batch create is a serial loop.

## Requirements

### R1: calculate_singularity_parallel()

Use rayon or manual thread splitting:
1. Split goal set into N chunks
2. Compute per-chunk position/tension contributions in parallel
3. Merge results into final IntentionSingularity
4. Golden path selection from merged tensions

### R2: scan_blockers_parallel()

1. Split goals into chunks
2. Scan each chunk for blockers in parallel
3. Merge and deduplicate blocker prophecies
4. Sort by confidence descending

### R3: progress_echoes_parallel()

1. Split goals into chunks
2. Detect echoes per chunk in parallel
3. Merge and sort by recency

### R4: create_goals_batch()

1. Validate all requests first (serial, fast)
2. Generate IDs and timestamps (serial, fast)
3. Insert all goals in one pass
4. Rebuild indexes once (not per-goal)

Note: True parallelism requires `&mut self` splitting which is complex.
A pragmatic approach is to optimize the serial path — validate upfront,
batch index rebuilds — rather than introduce unsafe parallel mutation.

## Acceptance Criteria

- [ ] Batch create rebuilds indexes once, not N times
- [ ] Functions are documented with their parallelism strategy
- [ ] Performance measurably better for N > 10 goals
