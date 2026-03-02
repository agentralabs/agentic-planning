# SPEC-04: Index Population & Validation Gaps

**Priority:** P1
**Files:** `indexes.rs`, `validation.rs`
**Estimated changes:** ~150 lines

## Problem

Two index HashMaps are defined but never populated. Validation is missing
critical bounds checks and structural integrity rules.

## Requirements

### R1: Populate goal_relationships index

In `indexes.rs`, the `goal_relationships` HashMap must be populated:
- On `rebuild()`: scan all goals for dependency links, parent-child, alliances
- On `add_goal()`: index the new goal's relationships
- On `update_goal()`: re-index if relationships changed
- On `remove_goal()`: clean up relationship entries

### R2: Populate commitment_entanglements index

- On `rebuild()`: scan all commitments for entanglement links
- On `add_commitment()`: index new entanglements
- On `fulfill/break_commitment()`: update entanglement state
- On `remove_commitment()`: clean up

### R3: Index dreams and federations in rebuild()

Currently `rebuild()` only processes goals, decisions, commitments.
Add indexing for:
- Dreams: by goal_id, by status
- Federations: by member, by sync status

### R4: Add missing validations

| Check | Rule |
|-------|------|
| emotional_weight bounds | Must be 0.0..=1.0 |
| self-dependency | Goal cannot depend on itself |
| stakeholder importance | Must be 0.0..=1.0 |
| deadline reachability | Deadline must be in the future |
| dead parent | Child's parent_id must reference existing goal |
| commitment weight | Must be 0.0..=1.0 |
| dream scenario count | At least 1 scenario required |
| federation member count | At least 2 members required |

### R5: Remove dead code

Remove `_keep_commitment_status` at validation.rs line 237.

## Acceptance Criteria

- [ ] `goal_relationships` populated on all goal mutations
- [ ] `commitment_entanglements` populated on all commitment mutations
- [ ] `rebuild()` processes all 5 entity types
- [ ] All 8 validation rules enforced
- [ ] Dead code removed
