# AgenticPlanning Python SDK

Thin Python wrapper around the `aplan` FFI library via `ctypes`.

## Install

```bash
pip install agentic-planning
```

Requires the `aplan` shared library (`.so`/`.dylib`/`.dll`) on your library path, or the `aplan` binary on PATH for CLI-mode fallback.

## Quick Start

```python
from agentic_planning import PlanningGraph

graph = PlanningGraph("project.aplan")
```

## Goals

```python
# Create a goal
goal = graph.create_goal("Ship v1", intention="Persistent intention infrastructure")
print(goal["id"])

# List all goals
goals = graph.list_goals()
for g in goals:
    print(f"{g['title']} — {g['status']} (momentum: {g['momentum']:.2f})")

# Update goal status
graph.activate_goal(goal["id"])
graph.complete_goal(goal["id"])

# Record progress
graph.record_progress(goal["id"], percentage=45, note="API endpoints done")

# Get goal with computed feelings
detail = graph.get_goal(goal["id"])
print(f"Urgency: {detail['feelings']['urgency']:.2f}")
print(f"Neglect: {detail['feelings']['neglect']:.2f}")
```

## Decisions

```python
# Create a decision linked to a goal
decision = graph.create_decision(
    title="Use PostgreSQL vs SQLite",
    goal_id=goal["id"],
    options=["PostgreSQL", "SQLite", "Both with abstraction layer"]
)

# Crystallize (choose an option)
graph.crystallize_decision(
    decision["id"],
    chosen="SQLite",
    reasoning="Simpler deployment, sufficient for planning state sizes"
)

# Query past decisions
history = graph.list_decisions(goal_id=goal["id"])
for d in history:
    print(f"{d['title']} — {d['status']}")
```

## Commitments

```python
# Create a commitment
commitment = graph.create_commitment(
    title="Deliver API docs by Friday",
    stakeholder="team-lead",
    stakeholder_weight=0.8,
    due_date="2026-03-07T17:00:00Z"
)

# Check at-risk commitments
at_risk = graph.list_commitments(status="at_risk")
for c in at_risk:
    print(f"AT RISK: {c['title']} (due: {c['due_date']})")

# Fulfill or break
graph.fulfill_commitment(commitment["id"])
# graph.break_commitment(commitment["id"], reason="Requirements changed")
```

## Singularity

```python
# Get the unified field view of all goals
singularity = graph.get_singularity()
print(f"Center: {singularity['center']}")
print(f"Themes: {singularity['themes']}")
print(f"Golden path: {singularity['golden_path']}")

# Check for tensions between goals
for tension in singularity["tensions"]:
    print(f"Tension: {tension['goal_a']} <-> {tension['goal_b']}")
```

## Blockers and Prophecy

```python
# Scan for blocked goals
blockers = graph.scan_blockers()
for b in blockers:
    print(f"{b['goal_title']} blocked by: {b['blocker_description']}")

# Listen for progress echoes
echoes = graph.listen_echoes()
for echo in echoes:
    print(f"Echo: {echo['source_goal']} -> {echo['affected_goal']}: {echo['effect']}")
```

## File Persistence

```python
# Save current state
graph.save()

# Load from file
graph = PlanningGraph("project.aplan")

# The .aplan file is portable — copy it anywhere
import shutil
shutil.copy("project.aplan", "/backup/project.aplan")
```

## Error Handling

All methods raise `PlanningError` on failure:

```python
from agentic_planning import PlanningError

try:
    graph.complete_goal("nonexistent-id")
except PlanningError as e:
    print(f"Error code: {e.code}")  # e.g., 4 (NotFound)
    print(f"Message: {e.message}")
```

Error codes match the FFI `AplanResult` enum:

| Code | Name | Meaning |
|------|------|---------|
| 0 | Ok | Success |
| 1 | NullPointer | Internal null pointer |
| 2 | InvalidUtf8 | Bad string encoding |
| 3 | EngineError | Engine-level failure |
| 4 | NotFound | Entity doesn't exist |
| 5 | ValidationError | Invalid input |
| 6 | IoError | File I/O failure |
| 7 | SerializationError | JSON error |
