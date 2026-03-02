---
status: stable
---

# Playbooks: Agent Integration

Ready-to-use patterns for integrating AgenticPlanning into AI agent workflows.

## 30-Second Quick Start

1. Install: `cargo install agentic-planning-cli`
2. Add to `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "agentic-planning": {
      "command": "aplan",
      "args": ["serve"]
    }
  }
}
```

3. First tool call: `planning_goal create "Ship v1.0" --intention "Deliver stable release"`

That's it. The agent now has persistent goal tracking.

## System Prompt Templates

### Minimal (~5 lines)

For agents that need basic goal tracking:

```
You have access to AgenticPlanning for tracking goals across sessions.
Use planning_goal to create, list, and complete goals.
Check planning_goal list at the start of each session to restore context.
Mark goals complete when done. Abandon goals that are no longer relevant.
```

### Standard (~15 lines)

For agents that need goals, decisions, and progress:

```
You have access to AgenticPlanning for structured planning across sessions.

Goals: Use planning_goal to create goals with priorities and deadlines.
Check goal feelings (urgency, neglect, confidence) to prioritize work.
Track physics (momentum, gravity) to understand progress dynamics.

Decisions: Use planning_decision to record choices with alternatives.
Crystallize decisions when you commit to a path. Shadow paths preserve
the options you didn't choose for later revisiting.

Progress: Use planning_progress to record incremental progress.
Check momentum to see if goals are accelerating or stalling.
Use planning_singularity collapse for a unified view of all goals.

At session start: planning_goal list to restore context.
At session end: record progress on active goals.
```

### Full (~25 lines)

For agents using the complete planning suite:

```
You have access to AgenticPlanning — a full planning engine with physics,
feelings, decisions, commitments, dreams, and multi-agent federation.

Goals: Create with planning_goal. Track lifecycle: Draft → Active → Blocked
→ Paused → Completed → Abandoned → Superseded → Reborn. Monitor physics
(momentum, gravity, inertia, energy) and feelings (urgency, neglect,
confidence, alignment, vitality). Decompose large goals into sub-goals.

Decisions: Record with planning_decision. Crystallize when choosing a path.
Shadow paths preserve alternatives. Use archaeology to search past decisions.
Use prophecy to predict outcomes before committing.

Commitments: Track promises with planning_commitment. Monitor entanglements
(Sequential, Parallel, Inverse, Resonant, Dependent). Check breaking cost
before renegotiating. Query at-risk and due-soon commitments.

Progress: Record with planning_progress. Check momentum and velocity.
Use blocker prophecy to predict obstacles. Listen for progress echoes
(ripple effects on related goals).

Dreams: Use planning_dream for forward-looking simulations. Generate
completion scenarios, predict obstacles, extract insights, seed sub-goals.

Singularity: Collapse with planning_singularity for a unified field view.
See goal positions, themes, tension lines, and the golden path.

Federation: Use planning_federate for multi-agent coordination. Share goals,
sync state, build consensus with planning_consensus.

Session protocol: Start with planning_goal list. End with progress records.
Use planning_context_log to capture reasoning throughout the session.
```

## Agent Workflow: Solo Planner

A single agent managing a project end-to-end:

1. **Session start** — `planning_goal list` to restore context. Check feelings for any rising neglect or falling confidence.
2. **Identify work** — `planning_singularity collapse` to see the unified view. The golden path shows the highest-impact next action.
3. **Make decisions** — `planning_decision create` with alternatives. Deliberate, then `crystallize` with reasoning.
4. **Track progress** — `planning_progress record` after each meaningful step. Note blockers immediately with `planning_goal block`.
5. **Dream forward** — `planning_dream create` on stalled goals to generate completion scenarios and obstacle predictions.
6. **Session end** — Record final progress. Log session context with `planning_context_log`.

## Agent Workflow: Decision Journal

For agents that need to maintain a searchable history of decisions:

1. **Before deciding** — `planning_decision create` with title, all options, and context. Use `prophecy` to predict outcomes.
2. **Deliberate** — Evaluate options. Record reasoning.
3. **Crystallize** — `planning_decision crystallize` with chosen path and full reasoning. Shadow paths preserve alternatives.
4. **Later review** — `planning_chain get` to see causal chains. `archaeology` to search past decisions by topic. `counterfactual` to explore what-if scenarios.
5. **Regret and recrystallize** — If context changes, `regret` a decision and `recrystallize` with new information.

## Agent Workflow: Commitment Tracker

For agents managing promises to stakeholders:

1. **Make promises** — `planning_commitment create` with stakeholder, deadline, and importance weight.
2. **Monitor health** — `planning_commitment list --filter at-risk` for commitments in danger. `--filter due-soon` for upcoming deadlines.
3. **Detect conflicts** — Entanglement analysis reveals parallel commitments that must progress together, inverse commitments that conflict, and sequential dependencies.
4. **Renegotiate carefully** — Check `breaking_cost` before breaking a commitment. Higher stakeholder importance means higher cost.
5. **Fulfill or release** — `planning_commitment fulfill` when done. `release` with explanation when circumstances change.

## Multi-Agent: Federation

For coordinating multiple agents on shared goals:

1. **Create federation** — `planning_federate create "Project Alpha"` with member agent identities.
2. **Share goals** — `planning_federate share` to publish goals to the federation. Members see shared goals in their planning state.
3. **Sync state** — `planning_federate sync` to propagate progress updates across members.
4. **Collective dreaming** — `planning_dream create --collective` to combine multiple agents' perspectives on obstacle prediction.
5. **Build consensus** — `planning_consensus propose` for decisions that affect all members. Members vote, and consensus is recorded.

## Tips

**Physics vs feelings:** Physics (momentum, gravity) measures objective progress dynamics. Feelings (urgency, neglect) measures subjective health. Use physics for prioritization algorithms. Use feelings for human-readable status reports.

**When to dream:** Dream on goals that have been active for a while but lack clear next steps. Dream when momentum is low. Dream before making irreversible decisions.

**When to collapse singularity:** At the start of a planning session to get oriented. After creating several new goals to see how they relate. Before making a high-impact decision to understand alignment.

**Context logging:** Use `planning_context_log` throughout sessions, not just at the end. It creates a searchable trail of reasoning that enriches future decision archaeology.
