---
status: stable
---

# Primary Problem Coverage

AgenticPlanning addresses four fundamental problems that AI agents face when working on non-trivial projects.

## Goal Drift Across Sessions

**The problem:** AI agents lose context between conversations. Goals stated in session 1 are forgotten by session 3. The agent starts each session fresh, reconstructing context from conversation history — an imprecise process that loses nuance. By session 5, original intentions have drifted as the agent's understanding of the project diverges from the initial vision. Estimated intention retention without structured planning: ~60% by session 3, declining further with each subsequent session.

**The solution:**

*Persistent `.aplan` files* store the complete planning state — goals, decisions, commitments, dreams, federations, and their indexes — in a single portable file. Opening the file at session start restores full context in under 1ms.

*Goal physics* track momentum over time. A goal that was making rapid progress in session 4 still has high momentum in session 5. A goal that stalled three sessions ago has near-zero momentum and high inertia — making it clear that it needs attention or honest abandonment.

*Feelings-based early warning* detects drift before it becomes critical. Rising neglect means the goal hasn't received attention. Falling confidence means progress trajectory has worsened. Declining alignment means the goal is diverging from related goals. These signals surface automatically — the agent doesn't need to remember to check.

*Reincarnation* handles the case where a goal was abandoned prematurely. If the need returns, the goal is reborn with karma — a full record of its previous life, why it was abandoned, and what changed. This prevents the cycle of creating, abandoning, and recreating the same goal without learning from history.

## Decision Amnesia

**The problem:** Agents make decisions but can't recall why, what alternatives existed, or what trade-offs were considered. When asked about a past choice, the agent either confabulates a justification or admits ignorance. When requirements change and a decision needs revisiting, there's no record of the alternatives that were evaluated. The agent starts from scratch, potentially re-evaluating options it already rejected for good reasons.

**The solution:**

*Crystallization* preserves the full decision context at the moment of choosing. The crystallized decision contains: the chosen path with reasoning, all shadow paths (unchosen alternatives with their trade-offs), the goals it affects, and its reversibility score.

*Shadow paths* keep unchosen options alive. When requirements change, the agent reviews shadow paths instead of re-evaluating from scratch. A shadow path that was rejected for "too expensive" becomes viable when budget increases. Recrystallization switches to the shadow path while preserving the full decision history.

*Archaeology* provides search across all past decisions. Query by topic ("What did we decide about the API versioning?"), by goal ("What decisions affected the authentication goal?"), or by time ("What was decided last week?").

*Prophecy* predicts outcomes before crystallizing. Based on historical patterns of similar decisions in the planning state, prophecy surfaces risks and likely consequences, informing better choices.

*Causal chains* track the downstream effects of decisions. When a decision leads to a blocked goal or a broken commitment, the chain makes the connection visible — enabling root cause analysis and better future decisions.

## Commitment Overload

**The problem:** Agents accumulate promises without tracking capacity or conflicts. Each commitment seems reasonable in isolation, but collectively they exceed what's achievable. Conflicting commitments aren't detected until deadlines pass. There's no visibility into which commitments matter most or what the cost of breaking one would be.

**The solution:**

*Weighted commitment inventory* tracks all active promises with stakeholder importance. A commitment to a critical stakeholder with weight 1.0 is more consequential than one to a peripheral stakeholder with weight 0.3. The inventory shows total commitment load weighted by importance.

*Entanglement detection* reveals relationships between commitments:
- **Sequential** dependencies: Commitment B can't start until Commitment A finishes.
- **Parallel** requirements: Commitments A and B must progress at the same rate.
- **Inverse** conflicts: Fulfilling A makes B harder or impossible.
- **Resonant** amplification: Progress on A accelerates B.
- **Dependent** chains: A depends on B's completion.

*At-risk and due-soon queries* surface commitments in danger before they fail. At-risk commitments have insufficient progress relative to their deadline. Due-soon commitments are approaching their deadline regardless of progress state.

*Breaking cost analysis* quantifies the consequences before breaking a promise. The cost factors in stakeholder importance, entanglement depth (how many other commitments are affected), and downstream goal impact. An agent can make an informed decision about which commitments to renegotiate when overloaded.

## Progress Blindness

**The problem:** No visibility into whether goals are actually advancing or stalled. Goals sit in "active" status indefinitely with no signal about their health. The agent reports optimistically until reality catches up. Stalls aren't detected until deadlines pass. There's no way to predict obstacles before they hit or understand how progress on one goal affects others.

**The solution:**

*Physics model* provides objective progress dynamics:
- **Momentum** measures rate of progress change. Positive momentum means accelerating. Negative means decelerating. Zero means stalled.
- **Gravity** models resource attraction based on priority. High-priority goals pull agent attention and effort toward them, just as massive objects pull orbiting bodies.

*Blocker prophecy* predicts obstacles before they materialize. If related goals have active blockers of a certain type, prophecy warns that this goal may encounter similar obstacles. If historical patterns show that goals in this domain tend to stall at a certain progress percentage, prophecy flags the risk.

*Progress echoes* propagate the effects of progress through the goal graph. When a sub-goal completes, the parent's momentum gets a boost. When a blocking dependency resolves, downstream goals become unblocked. When a goal stalls, its dependents feel increased gravitational drag. These ripple effects make the interconnected nature of progress visible.

*Singularity collapse* produces a unified field view of all goals. Instead of looking at goals individually, the singularity shows their positions relative to each other, identifies themes and clusters, reveals tension lines between conflicting goals, and highlights the golden path — the highest-impact sequence of actions across the entire planning state.
