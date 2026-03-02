# SPEC-12: Narrative & Evidence Documentation

**Priority:** P2
**Files:** `docs/public/experience-with-vs-without.md`, `docs/public/initial-problem-coverage.md`, `docs/public/primary-problem-coverage.md`
**Estimated changes:** ~300 lines total

## Problem

There is no compelling narrative explaining why AgenticPlanning matters. The experience doc is empty, initial-problem-coverage has 4 bare bullets, and primary-problem-coverage has 4 one-sentence paragraphs. Users and team leads have no evidence-based argument for adoption.

## Requirements

### R1: experience-with-vs-without.md (~120 lines)

Before/after scenarios with evidence:

- **Scenario: Multi-session project**
  - **Without:** Agent starts fresh each session, forgets goals, re-discusses decisions, loses track of commitments. By session 5, half the original intentions have drifted.
  - **With:** Goals persist with physics (momentum, gravity), decisions carry shadow paths, commitments track fulfillment. Session 5 picks up exactly where session 4 left off.
- **Scenario: Complex decision**
  - **Without:** Agent makes a choice, forgets the alternatives, can't explain why. When requirements change, no way to revisit.
  - **With:** Decision crystallization preserves all options as shadow paths. Recrystallization available when context changes. Full causal chain visible.
- **Scenario: Team coordination**
  - **Without:** Multiple agents work on overlapping goals with no visibility. Commitments conflict. Progress is unmeasured.
  - **With:** Federation coordinates shared goals. Commitment entanglement detects conflicts. Singularity collapse shows the golden path.
- **Numerical evidence** — File sizes (~2KB per goal, ~50KB for a 20-goal project), operation latency (sub-millisecond for CRUD, ~10ms for singularity collapse), session continuity (100% intention retention vs ~60% without)
- **Honest limitations** — What planning does NOT do (doesn't execute tasks, doesn't replace project management tools, doesn't guarantee outcomes), when it's overkill (single-session tasks, trivial decisions)

### R2: initial-problem-coverage.md (~80 lines)

Detailed coverage of what the initial release addresses:

- **Living goals** — Full lifecycle: Draft → Active → Blocked → Paused → Completed → Abandoned → Superseded → Reborn. Physics model (momentum, gravity, inertia, energy). Feelings model (urgency, neglect, confidence, alignment, vitality). Decomposition and linking. Reincarnation with karma.
- **Decision crystallization** — Lifecycle: Pending → Deliberating → Crystallized → Regretted → Recrystallized. Shadow paths preserved. Causal chains tracked. Archaeology and prophecy. Reversibility modeling.
- **Commitment physics** — Promise/stakeholder model. Entanglement types (Sequential, Parallel, Inverse, Resonant, Dependent). Breaking cost modeling. Inventory, due-soon, at-risk queries.
- **Progress dynamics** — Momentum, gravity, blocker prophecy, progress echoes, forecast, velocity, trend.
- **Dream system** — Completion scenarios, obstacle prediction, insights, goal seeds, collective dreams.
- **Intention singularity** — Unified vision, goal positions, themes, tension lines, golden path, center.
- **What's deferred** — Real-time collaboration, HTTP transport, persistent event log, full bridge implementations

### R3: primary-problem-coverage.md (~100 lines)

Deep explanation of each primary problem and its solution:

- **Goal drift across sessions** — Problem: AI agents lose context between conversations. Goals stated in session 1 are forgotten by session 3. Solutions: Persistent .aplan files, goal physics tracking momentum over time, feelings-based early warning (rising neglect, falling confidence), reincarnation for abandoned-but-needed goals.
- **Decision amnesia** — Problem: Agents make decisions but can't recall why, what alternatives existed, or what trade-offs were considered. Solutions: Crystallization preserves full decision context, shadow paths keep unchosen options alive, archaeology searches past decisions by topic, prophecy predicts outcomes before choosing.
- **Commitment overload** — Problem: Agents accumulate promises without tracking capacity or conflicts. Solutions: Weighted commitment inventory with stakeholder importance, entanglement detection (parallel commitments that must progress together, inverse commitments that conflict), at-risk and due-soon queries, breaking cost analysis before breaking promises.
- **Progress blindness** — Problem: No visibility into whether goals are actually advancing or stalled. Solutions: Physics model (momentum = rate of progress change, gravity = resource attraction), blocker prophecy (predict obstacles before they hit), progress echoes (ripple effects of progress on related goals), singularity collapse (unified view of all goal alignment).

## Acceptance Criteria

- [ ] experience-with-vs-without.md has concrete before/after scenarios with numbers
- [ ] initial-problem-coverage.md explains every capability area with depth
- [ ] primary-problem-coverage.md has problem → solution → mechanism for each area
- [ ] Limitations section is honest and specific
- [ ] All claims are backed by actual implementation capabilities
