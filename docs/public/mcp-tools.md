---
status: stable
---

# MCP Tools

AgenticPlanning exposes 13 MCP tools via the `planning_*` namespace.

## planning_goal

Manage the full lifecycle of living goals.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `create`, `list`, `show`, `activate`, `progress`, `complete`, `abandon`, `pause`, `resume`, `block`, `unblock`, `decompose`, `link`, `tree`, `feelings`, `physics`, `dream`, `reincarnate` |
| `goal_id` | string | varies | Goal UUID (required for most operations except `create`, `list`) |
| `title` | string | create | Goal title |
| `intention` | string | create | Why this goal matters |
| `description` | string | no | Goal description |
| `priority` | string | no | `critical`, `high`, `medium`, `low`, `someday` |
| `deadline` | integer | no | Unix timestamp |
| `parent` | string | no | Parent goal UUID |
| `tags` | array | no | String tags |
| `percentage` | number | progress | Progress 0.0–1.0 |
| `note` | string | no | Progress or completion note |
| `reason` | string | varies | Reason for abandon/pause/block |
| `blocker_id` | string | unblock | Blocker UUID to resolve |
| `resolution` | string | unblock | How the blocker was resolved |
| `goal_b` | string | link | Second goal UUID for linking |
| `relationship` | string | no | Relationship type for link |
| `lessons` | array | reincarnate | Lessons from previous life |

**Returns:** JSON object with operation result. Create returns the new goal; list returns an array; show returns full goal detail with feelings/physics.

**Example:**

```json
{
  "tool": "planning_goal",
  "arguments": {
    "operation": "create",
    "title": "Ship v2.0",
    "intention": "Deliver the next major release to users",
    "priority": "high",
    "tags": ["release", "q2"]
  }
}
```

---

## planning_decision

Crystallize decisions with shadow path preservation.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `create`, `option`, `crystallize`, `show`, `shadows`, `chain`, `archaeology`, `prophecy`, `counterfactual`, `regret`, `recrystallize` |
| `decision_id` | string | varies | Decision UUID |
| `question` | string | create | The decision question |
| `context` | string | no | Decision context |
| `name` | string | option | Option name |
| `description` | string | no | Option description |
| `pros` | array | no | Pros of option |
| `cons` | array | no | Cons of option |
| `chosen` | string | crystallize | Chosen path UUID |
| `reasoning` | string | no | Decision rationale |
| `artifact` | string | archaeology | Artifact to investigate |

**Returns:** JSON object. Create returns the decision; crystallize returns the crystallized decision with shadow paths; prophecy returns projected outcomes.

---

## planning_commitment

Track weighted commitments to stakeholders.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `create`, `list`, `show`, `fulfill`, `break`, `renegotiate`, `entangle`, `inventory`, `due_soon`, `at_risk` |
| `commitment_id` | string | varies | Commitment UUID |
| `promise` | string | create | What is being promised |
| `stakeholder` | string | create | Who receives the promise |
| `due` | integer | no | Due date as Unix timestamp |
| `goal_id` | string | no | Linked goal UUID |
| `importance` | number | no | Stakeholder importance 0.0–1.0 (default: 0.6) |
| `how_delivered` | string | fulfill | Description of delivery |
| `reason` | string | break/renegotiate | Why breaking or renegotiating |
| `new_promise` | string | renegotiate | Updated promise text |
| `commitment_b` | string | entangle | Second commitment UUID |
| `entanglement_type` | string | no | `sequential`, `parallel`, `inverse`, `resonant`, `dependent` |
| `strength` | number | no | Coupling strength 0.0–1.0 (default: 0.8) |
| `within_days` | integer | no | Due-soon window (default: 7) |

**Returns:** JSON object. Inventory returns `CommitmentInventory` with total weight and overload status.

---

## planning_progress

Query progress physics across goals.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `momentum`, `gravity`, `blockers`, `echoes`, `forecast`, `velocity`, `trend` |
| `goal_id` | string | no | Focus on specific goal |

**Returns:** JSON. Momentum returns physics values; blockers returns `BlockerProphecy` array; echoes returns `ProgressEcho` array; forecast returns estimated completion.

---

## planning_singularity

Analyze the intention singularity — the convergence point of all goals.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `collapse`, `position`, `path`, `tensions`, `themes`, `center`, `vision` |
| `goal_id` | string | position | Goal to position in the singularity |

**Returns:** JSON. Collapse returns full `IntentionSingularity`; position returns `IntentionPosition`; tensions returns `TensionLine` array; center returns `IntentionCenter`.

---

## planning_dream

Generate and interpret predictive dream surfaces.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `goal`, `collective`, `interpret`, `insights`, `accuracy`, `history` |
| `goal_id` | string | goal | Goal to dream about |
| `dream_id` | string | interpret/accuracy | Dream UUID |
| `federation_id` | string | no | Federation for collective dreams |

**Returns:** JSON. Goal returns a `Dream` with scenario, obstacles, and insights; collective returns `CollectiveDream` with synthesis.

---

## planning_counterfactual

Project and compare alternative decision paths.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `project`, `compare`, `learn`, `timeline` |
| `decision_id` | string | yes | Decision UUID |
| `path_id` | string | project | Path to project |
| `path_a` | string | compare | First path UUID |
| `path_b` | string | compare | Second path UUID |

**Returns:** JSON. Project returns `CounterfactualProjection`; compare returns side-by-side analysis; learn returns extracted lessons.

---

## planning_chain

Trace and visualize decision causality chains.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `trace`, `cascade`, `roots`, `leaves`, `visualize` |
| `decision_id` | string | yes | Decision UUID to trace from |

**Returns:** JSON. Trace returns `DecisionChain` with descendants and `CascadeAnalysis`; roots returns root decisions; leaves returns terminal consequences.

---

## planning_consensus

Run multi-stakeholder consensus workflows.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `start`, `round`, `synthesize`, `vote`, `status`, `crystallize` |
| `decision_id` | string | yes | Decision UUID |
| `stakeholders` | array | start | Stakeholder UUIDs |
| `stakeholder` | string | round/vote | Individual stakeholder UUID |
| `position` | string | round | Stakeholder position |
| `option` | string | vote/crystallize | Chosen option |
| `rationale` | string | crystallize | Decision rationale |

**Returns:** JSON. Status returns consensus progress; synthesize returns merged positions; crystallize finalizes the decision.

---

## planning_federate

Coordinate goals across multiple agents.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `create`, `join`, `sync`, `handoff`, `status`, `members` |
| `federation_id` | string | varies | Federation UUID |
| `goal_id` | string | create | Root goal UUID |
| `agent_id` | string | create/join/handoff | Agent identifier |
| `coordinator` | string | no | Coordinator agent ID |

**Returns:** JSON. Status returns `Federation` with members and sync status; members returns `FederationMember` array.

---

## planning_metamorphosis

Detect and manage goal transformations.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `detect`, `approve`, `history`, `predict`, `stage` |
| `goal_id` | string | yes | Goal UUID |
| `title` | string | approve | New title after transformation |
| `description` | string | approve | New description |
| `change_type` | string | approve | `expansion`, `contraction`, `pivot`, `refinement` |
| `factor` | number | approve | Magnitude 0.0–1.0 |
| `reason` | string | approve | Reason for change |
| `new_direction` | string | no | For pivot changes |
| `clarification` | string | no | For refinement changes |

**Returns:** JSON. Detect returns `MetamorphosisSignal`; predict returns `MetamorphosisPrediction`; stage returns current `MetamorphicStage`.

---

## planning_workspace

Manage planning workspaces (isolated contexts).

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `operation` | string | yes | One of: `create`, `switch`, `list`, `compare`, `merge`, `delete` |
| `name` | string | create/switch/delete | Workspace name |
| `left` | string | compare | Left workspace name |
| `right` | string | compare | Right workspace name |
| `source` | string | merge | Source workspace |
| `target` | string | merge | Target workspace |

**Returns:** JSON. Compare returns diff; merge returns `MergeReport` with counts.

---

## planning_context_log

Log the intent and context behind a planning action. Captures WHY, not just what.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `intent` | string | yes | Why you are performing this planning action |
| `finding` | string | no | What you found or concluded |
| `topic` | string | no | Category (e.g., `goal-review`, `decision-analysis`) |

**Returns:** JSON confirmation with logged entry ID.
