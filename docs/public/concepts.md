---
status: stable
---

# Concepts

Core domain concepts in AgenticPlanning.

## Goal Physics

Goals are treated as physical objects with measurable dynamics.

**GoalPhysics** fields:

| Field | Type | Description |
|-------|------|-------------|
| `momentum` | f64 | Rate of progress change — high momentum means accelerating progress |
| `gravity` | f64 | Attractiveness pulling attention and resources toward the goal |
| `inertia` | f64 | Resistance to change — high inertia goals are hard to stop or redirect |
| `energy` | f64 | Available energy for the goal — depletes under sustained effort |

Physics are recalculated from progress history, blocker state, and time since last activity. A goal with high momentum and low inertia is making rapid, adaptable progress. A goal with high gravity but low momentum is important but stalled.

---

## Goal Feelings

Goals have an emotional state reflecting their health.

**GoalFeelings** fields:

| Field | Type | Description |
|-------|------|-------------|
| `urgency` | f64 | How pressing the goal is — rises as deadlines approach |
| `neglect` | f64 | How long since the goal received attention — rises with inactivity |
| `confidence` | f64 | Likelihood of success based on progress trajectory |
| `alignment` | f64 | How well the goal aligns with the intention singularity |
| `vitality` | f64 | Overall health score combining all factors |

Feelings serve as an early warning system. Rising neglect or falling confidence signal goals that need intervention before they become blocked.

---

## Decision Crystallization

Decisions follow a crystallization lifecycle inspired by phase transitions.

**Lifecycle:** `Pending` → `Deliberating` → `Crystallized` → (optionally) `Regretted` → `Recrystallized`

**Key concepts:**

- **DecisionQuestion** — The question, context, and constraints framing the decision
- **DecisionPath** — A candidate option with name, description, pros, cons, estimated effort, and risk
- **Crystallization** — The act of choosing a path, with reasoning and confidence
- **Shadow paths** — Unchosen options preserved as `CrystalShadow` with rejection reason and counterfactual projection
- **Reversibility** — Each decision tracks whether it can be reversed, at what cost, and within what time window
- **Decision chain** — Decisions cause other decisions via `CausalLink` (enables, constrains, suggests, requires, precludes)

Shadow paths are never deleted. They remain as counterfactual possibilities that can inform future decisions or be resurrected via recrystallization.

---

## Commitment Entanglement

Commitments can be entangled — coupled such that changes in one affect the other.

**EntanglementType** variants:

| Type | Description |
|------|-------------|
| `Sequential` | One must complete before the other can start |
| `Parallel` | Both must progress together |
| `Inverse` | Fulfilling one makes the other harder |
| `Resonant` | Progress on one amplifies the other |
| `Dependent` | One cannot exist without the other |

Each entanglement has a `strength` (0.0–1.0) governing how strongly changes propagate.

**Breaking cost** is modeled explicitly with `trust_damage`, `relationship_impact`, `reputation_cost`, `energy_to_break`, and `cascading_effects`.

---

## Dream System

Dreams are predictive simulations of goal completion.

**Dream** contains:

- **CompletionScenario** — A vision of the completed goal: what the world looks like, how it feels, stakeholder reactions
- **DreamObstacle** — Predicted obstacles with severity, timing, and mitigation strategies
- **DreamInsight** — Actionable insights discovered during the dream
- **GoalSeed** — New goals discovered by dreaming about existing ones
- **DreamAccuracy** — Post-hoc assessment comparing predictions to actual outcomes

**Collective dreams** synthesize individual dreams from federation members into a `DreamSynthesis` with unified vision, themes, conflicts, and emergent goals.

---

## Federation Model

Federations enable multi-agent coordination around shared goals.

**Federation** contains:

- A root `goal_id` that the federation forms around
- `FederationMember` entries with agent ID, owned sub-goals, progress, and status
- A `coordinator` agent responsible for sync
- `SyncStatus` tracking: `Synced`, `Pending`, `Conflict`, `Error`
- Collective dreams combining member perspectives

Members can join, sync state, and hand off coordination. Each member tracks their own progress on owned sub-goals.

---

## Metamorphosis and Reincarnation

Goals can transform (metamorphosis) or die and be reborn (reincarnation).

### Metamorphosis

Goals evolve through `MetamorphicStage` transitions:

| ScopeChange | Description |
|-------------|-------------|
| `Expansion` | Goal scope grows (factor > 1.0) |
| `Contraction` | Goal scope shrinks |
| `Pivot` | Goal changes direction entirely |
| `Refinement` | Goal becomes clearer without changing scope |

The engine can `detect` when a goal should transform (via `MetamorphosisSignal`) and `predict` future transformations. Each stage preserves the `invariant_soul` — the core intention that survives transformation.

### Reincarnation

When a goal dies (completed, abandoned), its soul is archived as `GoalSoulArchive` with:

- **GoalKarma** — Lessons from failures, near-successes, and requirements for success
- **PreviousLife** — Link to original goal, death cause, and lessons learned
- **Reincarnation potential** — Score indicating how likely the goal is to return
- **Trigger conditions** — What would cause the goal to be reborn

Reincarnating a goal creates a new goal carrying forward the karma and lessons of its previous life.

---

## Intention Singularity

The singularity is the convergence point where all goals, decisions, and commitments align.

**IntentionSingularity** contains:

| Field | Description |
|-------|-------------|
| `unified_vision` | A synthesized statement of where everything is heading |
| `goal_positions` | Each goal's position in the singularity field (centrality, alignment angle, gravitational pull, drift risk) |
| `themes` | Emergent themes across all goals |
| `tension_lines` | Conflicts between goal pairs with magnitude and reason |
| `golden_path` | Ordered sequence of goals representing the optimal path |
| `center` | The singularity center with urgency, confidence, and momentum |

**IntentionPosition** places each goal in a field:

- `centrality` — How central the goal is to the overall vision
- `alignment_angle` — How well the goal aligns with the unified direction
- `gravitational_pull` — How much the goal pulls the singularity toward itself
- `drift_risk` — Risk of the goal drifting away from alignment

The singularity collapses when goals are sufficiently aligned, producing the golden path — the sequence of actions that best serves the unified vision.
