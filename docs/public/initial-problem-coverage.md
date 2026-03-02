---
status: stable
---

# Initial Problem Coverage

The initial release of AgenticPlanning covers six capability areas and one unifying system.

## Living Goals

Full goal lifecycle with eight states: Draft, Active, Blocked, Paused, Completed, Abandoned, Superseded, Reborn.

**Physics model:** Every goal carries four physics properties computed from its history:
- **Momentum** — Rate of progress change. High momentum means accelerating; low momentum means stalling.
- **Gravity** — Resource attraction. High-priority goals with strong stakeholder weight pull resources toward them.
- **Inertia** — Resistance to change. Goals with deep dependency trees and long histories resist status transitions.
- **Energy** — Overall vitality. Combines momentum, priority, and emotional weight into a single score.

**Feelings model:** Five emotional properties computed on access (not stored):
- **Urgency** — Derived from priority and deadline proximity. Critical goals with approaching deadlines feel maximally urgent.
- **Neglect** — Time since last progress update. Rises automatically if the goal isn't touched.
- **Confidence** — Based on progress trajectory. Consistent forward movement builds confidence; stalls erode it.
- **Alignment** — How well this goal aligns with related goals. Measured by relationship quality and shared momentum direction.
- **Vitality** — Composite health score. A goal with high momentum, low neglect, and high confidence feels vital.

**Decomposition and linking:** Goals can be broken into sub-goals (parent-child relationships) and linked with typed relationships: Supports, Blocks, Enables, Depends, Conflicts, Complements.

**Reincarnation:** Abandoned goals can be reborn with karma — a history of what happened in the previous life. The soul archive preserves this across reincarnation cycles.

## Decision Crystallization

Five-state lifecycle: Pending, Deliberating, Crystallized, Regretted, Recrystallized.

**Shadow paths:** When a decision is crystallized, the unchosen alternatives are preserved as shadow paths. They remain searchable and available for recrystallization if context changes.

**Causal chains:** Decisions link to the goals they affect and to other decisions they enabled or constrained. The chain is traversable forward (what did this decision cause?) and backward (what caused this decision?).

**Archaeology and prophecy:** Search past decisions by topic, time, or goal. Predict outcomes before crystallizing based on related historical decisions.

**Reversibility:** Each decision carries a reversibility score. Highly reversible decisions need less deliberation. Irreversible decisions demand full evaluation of all paths.

## Commitment Physics

**Promise/stakeholder model:** Each commitment has a description, a stakeholder (who it's made to), an importance weight, and a deadline.

**Entanglement types:** Five types describe how commitments interact:
- **Sequential** — One must complete before the next can start.
- **Parallel** — Must progress together at roughly the same rate.
- **Inverse** — Fulfilling one makes the other harder or impossible.
- **Resonant** — Progress on one amplifies progress on the other.
- **Dependent** — One commitment depends on another's completion.

**Breaking cost:** Computed from stakeholder importance, entanglement depth, and downstream impact. Higher cost means higher consequences for breaking the promise.

**Queries:** Inventory (all commitments with health), due-soon (approaching deadlines), at-risk (falling behind), by-stakeholder, by-goal.

## Progress Dynamics

- **Momentum** — Computed from progress history: rate of change in progress over time.
- **Gravity** — Priority-weighted pull. Higher priority goals exert stronger gravitational pull on agent attention.
- **Blocker prophecy** — Predicts likely obstacles based on current blockers of related goals and historical blocking patterns.
- **Progress echoes** — When a goal makes progress, related goals feel ripple effects. A sub-goal completing boosts the parent's momentum. A dependency resolving unblocks downstream goals.
- **Forecast** — Projects completion date based on current velocity and remaining work.
- **Velocity and trend** — Rolling average of progress rate, with trend direction (accelerating, decelerating, or steady).

## Dream System

Forward-looking simulations tied to goals:

- **Completion scenarios** — Generate possible paths to completion with estimated effort and probability.
- **Obstacle prediction** — Identify likely blockers before they materialize, based on goal physics and related goal state.
- **Insights** — Extract patterns from the planning state that aren't obvious from individual goals.
- **Sub-goal seeds** — Dreams can propose new sub-goals that would accelerate the parent.
- **Collective dreams** — In federation mode, multiple agents contribute perspectives to a shared dream.

## Intention Singularity

The singularity collapse produces a unified field view of all planning state:

- **Goal positions** — Where each goal sits in a conceptual space based on priority, momentum, and alignment.
- **Themes** — Clusters of related goals that form natural groupings.
- **Tension lines** — Conflicts and trade-offs between goals that need resolution.
- **Golden path** — The highest-impact sequence of actions across all goals.
- **Center** — The central intention that all goals orbit around.

## What's Deferred

The initial release does not include:

- **Real-time collaboration** — Multi-user editing of the same `.aplan` file is not supported. Federation handles multi-agent coordination instead.
- **HTTP transport** — The MCP server runs over stdio. HTTP mode is planned but not yet implemented.
- **Persistent event log** — Mutations are saved as snapshots, not as an append-only event log.
- **Full bridge implementations** — Bridge traits are defined with NoOp defaults. Real implementations connecting to sister systems are available separately.
