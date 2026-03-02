---
status: stable
---

# MCP Resources

AgenticPlanning exposes 9 read-only resources via the `planning://` URI scheme. All return `application/json`.

## planning://goals

List all goals in the current workspace.

**Response:** JSON array of `Goal` objects with full detail (status, feelings, physics, blockers, progress).

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Ship v2.0",
    "status": "Active",
    "priority": "High",
    "progress": { "percentage": 0.45, "velocity": 0.02 },
    "feelings": { "urgency": 0.7, "confidence": 0.6 },
    "physics": { "momentum": 0.5, "gravity": 0.3 }
  }
]
```

---

## planning://goals/{id}

Get a single goal by UUID.

**URI parameters:** `{id}` — Goal UUID

**Response:** Full `Goal` object including soul, relationships, blockers, decisions, commitments, dreams, metamorphosis history, and provenance.

---

## planning://decisions

List all decisions.

**Response:** JSON array of `Decision` objects with question, status, chosen path, shadow paths, reasoning, and reversibility.

---

## planning://commitments

List all commitments.

**Response:** JSON array of `Commitment` objects with promise, stakeholder, status, weight, entanglements, and breaking cost.

---

## planning://singularity

Get the current intention singularity analysis.

**Response:** `IntentionSingularity` object containing:

```json
{
  "unified_vision": "...",
  "goal_positions": { "<goal_id>": { "centrality": 0.8, "alignment_angle": 0.3 } },
  "themes": ["shipping", "quality"],
  "tension_lines": [{ "a": "<id>", "b": "<id>", "magnitude": 0.6, "reason": "..." }],
  "golden_path": ["<id1>", "<id2>"],
  "center": { "urgency": 0.5, "confidence": 0.7, "momentum": 0.4 }
}
```

---

## planning://status

Get a planning status overview.

**Response:** Summary object with counts and health indicators:

```json
{
  "active_goals": 5,
  "blocked_goals": 1,
  "active_commitments": 8,
  "total_commitments": 12,
  "total_weight": 4.8,
  "is_overloaded": false,
  "pending_decisions": 2,
  "active_federations": 1
}
```

---

## planning://dreams/{id}

Get dream detail by UUID.

**URI parameters:** `{id}` — Dream UUID

**Response:** Full `Dream` object with scenario (vision, feeling, world changes, stakeholder reactions), obstacles, insights, discovered goals, and accuracy assessment.

---

## planning://consensus/{id}

Get consensus session state.

**URI parameters:** `{id}` — Decision UUID that has a consensus session

**Response:** Consensus state with stakeholder positions, votes, synthesis result, and crystallization status.

---

## planning://workspace/{id}

Get workspace summary.

**URI parameters:** `{id}` — Workspace name or UUID

**Response:** Workspace metadata with goal count, decision count, commitment count, and last-modified timestamp.
