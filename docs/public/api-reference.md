---
status: stable
---

# API Reference

Rust API for the `agentic-planning` crate.

## PlanningEngine

The core engine managing all planning state. Supports in-memory and file-backed modes.

### Construction

```rust
use agentic_planning::PlanningEngine;

// In-memory engine
let engine = PlanningEngine::new();

// File-backed engine
let engine = PlanningEngine::from_file("planning.aplan")?;
```

### Goal Methods

```rust
// Create a goal
let goal = engine.create_goal(CreateGoalRequest {
    title: "Ship v2.0".into(),
    description: "Next major release".into(),
    intention: "Deliver value to users".into(),
    priority: Some(Priority::High),
    deadline: Some(Timestamp(1735689600)),
    ..Default::default()
})?;

// List goals with optional filter
let goals = engine.list_goals(Some(GoalFilter {
    status: Some(vec![GoalStatus::Active]),
    priority: Some(vec![Priority::High, Priority::Critical]),
    limit: Some(10),
    ..Default::default()
}))?;

// Get a specific goal
let goal = engine.get_goal(goal_id)?;

// Lifecycle transitions
engine.activate_goal(goal_id)?;
engine.update_progress(goal_id, 0.5, Some("Halfway there"))?;
engine.pause_goal(goal_id, Some("Waiting on dependency"))?;
engine.resume_goal(goal_id)?;
engine.complete_goal(goal_id, Some("All criteria met"))?;
engine.abandon_goal(goal_id, "No longer relevant")?;

// Blockers
engine.block_goal(goal_id, "API dependency unavailable", 0.8)?;
engine.unblock_goal(goal_id, blocker_id, "Dependency released")?;

// Structure
engine.decompose_goal(goal_id, sub_goal_specs)?;
engine.link_goals(goal_a, goal_b, GoalRelationship::Alliance { .. })?;
let tree = engine.goal_tree(Some(root_id))?;

// Analysis
let feelings = engine.goal_feelings(goal_id)?;
let physics = engine.goal_physics(goal_id)?;

// Reincarnation
let new_goal = engine.reincarnate_goal(goal_id, ReincarnationUpdates {
    title: Some("Ship v2.1".into()),
    lessons_learned: Some(vec!["Scope too large".into()]),
    ..Default::default()
})?;
```

### Decision Methods

```rust
// Create and populate
let decision = engine.create_decision(CreateDecisionRequest {
    question: "Which database?".into(),
    context: Some("Need sub-10ms reads".into()),
    ..Default::default()
})?;

engine.add_option(decision.id, DecisionPath {
    name: "PostgreSQL".into(),
    pros: vec!["Mature".into()],
    cons: vec!["Slower reads".into()],
    ..Default::default()
})?;

// Crystallize (choose a path)
engine.crystallize(decision.id, chosen_path_id, "Best fit for requirements")?;

// Analysis
let shadows = engine.decision_shadows(decision.id)?;
let chain = engine.decision_chain(decision.id)?;
let archaeology = engine.decision_archaeology("database choice")?;
let prophecy = engine.decision_prophecy(question)?;
let regret = engine.decision_regret(decision.id)?;

// Revise
engine.recrystallize(decision.id, new_path_id, "Requirements changed")?;
```

### Commitment Methods

```rust
let commitment = engine.create_commitment(CreateCommitmentRequest {
    promise: Promise {
        description: "Deliver API docs".into(),
        deliverables: vec!["OpenAPI spec".into()],
        conditions: vec![],
    },
    stakeholder: Stakeholder {
        name: "Engineering".into(),
        role: "consumer".into(),
        importance: 0.8,
        ..Default::default()
    },
    due: Some(Timestamp(1735689600)),
    goal: Some(goal_id),
})?;

engine.fulfill_commitment(commitment.id, "Docs published")?;
engine.break_commitment(commitment.id, "Scope changed")?;
engine.renegotiate(commitment.id, new_promise, "Timeline shifted")?;
engine.entangle(commitment.id, other_id, EntanglementType::Parallel, 0.8)?;

let inventory = engine.commitment_inventory()?;
let due_soon = engine.commitments_due_soon(7)?;
let at_risk = engine.commitments_at_risk()?;
```

### Progress & Query Methods

```rust
let momentum = engine.momentum(Some(goal_id))?;
let gravity = engine.gravity()?;
let blockers = engine.predict_blockers()?;   // Vec<BlockerProphecy>
let echoes = engine.listen_echoes()?;         // Vec<ProgressEcho>
let forecast = engine.forecast(goal_id)?;
let velocity = engine.velocity()?;
let trend = engine.trend()?;
```

### Singularity Methods

```rust
let singularity = engine.collapse_singularity()?;
let position = engine.position_goal(goal_id)?;
let path = engine.golden_path()?;
let tensions = engine.tensions()?;
let themes = engine.themes()?;
let center = engine.intention_center()?;
let vision = engine.unified_vision()?;
```

### Dream Methods

```rust
let dream = engine.dream_goal(goal_id)?;
let collective = engine.dream_collective(Some(federation_id))?;
let interpreted = engine.interpret_dream(dream_id)?;
let insights = engine.dream_insights(Some(goal_id))?;
let accuracy = engine.dream_accuracy(dream_id)?;
let history = engine.dream_history(Some(goal_id))?;
```

### Persistence

```rust
engine.save()?;                          // Save to configured path
engine.load("other.aplan")?;            // Load from path
```

---

## Error Handling

All engine methods return `Result<T, PlanningError>`.

```rust
pub enum PlanningError {
    NotFound(String),
    InvalidTransition(String),
    ValidationError(String),
    IoError(std::io::Error),
    SerializationError(String),
    EngineError(String),
}
```

---

## Key Types

See [concepts.md](concepts.md) for the domain model. Key request/response types:

- `CreateGoalRequest` — Fields for new goal creation
- `CreateDecisionRequest` — Fields for new decision
- `CreateCommitmentRequest` — Fields for new commitment
- `GoalFilter` — Filter criteria for goal listing (status, priority, tags, deadline, progress range, limit)
- `ReincarnationUpdates` — Optional overrides for reincarnated goal
- `MergeReport` — Result of workspace merge (counts per entity type)

See [ffi-reference.md](ffi-reference.md) for the C FFI interface.
