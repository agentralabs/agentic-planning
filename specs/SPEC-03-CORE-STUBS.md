# SPEC-03: Core Engine Stub Implementation

**Priority:** P1 — Fake data returned to callers
**Files:** `query_engine.rs`, `write_engine.rs`, `lib.rs`
**Estimated changes:** ~400 lines

## Problem

12 core functions return hardcoded/fake data instead of computing real results.

## Requirements

### R1: build_chain_recursive() — Real causality types

Instead of always `CausalityType::Enables`, determine the actual relationship:
- If goal B is in goal A's dependencies → `Enables`
- If goal B blocks goal A (via blockers) → `Blocks`
- If goals share a dependency → `Amplifies`
- If goals compete for same resource → `Dampens`
- Default only if no relationship detected → `Enables`

### R2: decision_archaeology() — Real assessment

- `was_reasonable`: Compare decision's chosen option against current state.
  If the outcome aligns with stated success_criteria, mark reasonable.
  If most criteria are unmet and an alternative had better alignment, mark unreasonable.
- `insights`: Generate from patterns:
  - "Decision made under time pressure" if deadline was < 24h away
  - "Alternative X had higher alignment score" if true
  - "Decision reversed N times" if corrections exist

### R3: scan_blocker_prophecy() — Computed predictions

- `prediction_confidence`: Based on blocker type recurrence. If this type
  of blocker has appeared before on similar goals, higher confidence.
  Default 0.5 for novel blockers, scaled up by historical match.
- `days_until_materialization`: Based on goal deadline proximity and
  blocker severity. Compute from `(deadline - now) * severity_factor`.

### R4: generate_projected_timeline() — Real projections

Use goal physics (momentum, energy, gravity) to project future states:
1. Current momentum determines velocity
2. Gravity from dependencies determines drag
3. Energy determines sustainability
4. Project state at intervals: 25%, 50%, 75%, 100% of remaining time

### R5: check_success_criteria() — Actual evaluation

For each criterion:
- Parse the criterion text for measurable conditions
- Check if the goal's current state satisfies the condition
- Return `achieved` only if condition is met
- Include partial progress tracking

### R6: calculate_chain_bonus() — Commitment chain analysis

- Traverse the commitment's entanglements
- If the commitment is part of a fulfilled chain, bonus = `0.05 * chain_length`
- If part of a broken chain, bonus = `-0.02 * broken_count`
- Cap between -0.1 and 0.3

### R7: pause_goal() / break_commitment() — Store reasons

- `pause_goal()`: Store reason in goal metadata or a new `pause_reason` field
- `break_commitment()`: Store reason in commitment metadata

### R8: renegotiate_commitment() — Real negotiation

Instead of always `accepted: true`:
- Check if new terms are within acceptable bounds (weight change < 50%, deadline extension < 2x)
- If terms are extreme, set `accepted: false` with explanation
- Track negotiation history

### R9: synthesize_vision() — Meaningful synthesis

- Group goals by relationship (parent-child clusters)
- Identify dominant themes across goals
- Detect conflicts between goal groups
- Generate a coherent vision statement from themes

### R10: find_tensions() — Multi-dimensional tension detection

Beyond urgency divergence, check:
- Resource conflicts (goals needing same stakeholders)
- Timeline conflicts (overlapping deadlines with dependencies)
- Value conflicts (goals with opposing success criteria)
- Energy conflicts (total energy demand > available capacity)

## Acceptance Criteria

- [ ] No function returns hardcoded data
- [ ] All computations use actual engine state
- [ ] Existing tests still pass
- [ ] New tests cover each fixed function
