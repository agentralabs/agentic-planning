---
status: stable
---

# Experience: With vs Without Planning

Concrete before/after scenarios showing what changes when agents have structured planning.

## Scenario: Multi-Session Project

**Without AgenticPlanning:**

Session 1: Agent discusses goals, makes decisions, starts work. Session 2: Agent has no memory of session 1. User re-explains the project. Agent re-discusses the same decisions. Session 3: Half the original intentions have drifted — the agent pursues slightly different goals because context was lost. By session 5, the project has diverged significantly from the original vision. Estimated intention retention: ~60% by session 3, ~30% by session 5.

**With AgenticPlanning:**

Session 1: Agent creates goals with priorities and deadlines. Records decisions with reasoning. Tracks commitments. Session 2: `planning_goal list` restores full context in one call. Goals carry physics (momentum from session 1 progress) and feelings (low neglect because progress was recent). Session 5: The `.aplan` file contains the complete project state — every goal, decision, and commitment from all prior sessions. Intention retention: 100%. The agent picks up exactly where session 4 left off.

**Numbers:** A 20-goal project produces a ~60 KB `.aplan` file. Loading it takes under 1ms. The agent regains full project context instantly instead of spending the first 5-10 minutes of each session reconstructing it from conversation history.

## Scenario: Complex Decision

**Without AgenticPlanning:**

Agent evaluates three architecture options and picks option B. Two weeks later, requirements change. The agent can't recall what options A and C were, why B was chosen, or what trade-offs were considered. It makes a new decision from scratch — potentially re-evaluating options it already rejected for good reasons. No audit trail. No ability to revisit.

**With AgenticPlanning:**

Agent creates a decision with three paths. Records reasoning for each. Crystallizes option B with full justification. Shadow paths preserve options A and C with their trade-offs intact. Two weeks later, when requirements change, the agent uses decision archaeology to find the original decision, reviews shadow paths, and recrystallizes with option C — carrying forward all original analysis. The causal chain shows exactly how the first decision influenced subsequent work, making the switch safe and informed.

**Numbers:** Decision crystallization adds ~1.5 KB per decision. A project with 15 decisions uses ~22 KB. Archaeology search over 15 decisions completes in under 1ms.

## Scenario: Team Coordination

**Without AgenticPlanning:**

Three agents work on related features. Agent A makes a commitment that conflicts with Agent B's timeline. Agent C duplicates work that Agent A already completed. No one knows the overall status. Progress meetings are manual synchronization exercises that cost time and still miss conflicts.

**With AgenticPlanning:**

A federation coordinates all three agents. Shared goals give everyone visibility into the full picture. Commitment entanglement analysis detects that Agent A's promise conflicts with Agent B's timeline (inverse entanglement). Agent C sees Agent A's completed sub-goal before starting redundant work. Singularity collapse shows the golden path — the optimal sequence for all three agents to reach their goals without conflicts.

## Scenario: Stalled Progress

**Without AgenticPlanning:**

A goal has been "in progress" for two weeks with no measurable advancement. The agent keeps reporting it as active because there's no mechanism to detect stalling. The user discovers the stall manually weeks later.

**With AgenticPlanning:**

Goal physics detect the stall: momentum drops to near-zero, gravity pulls the goal down the priority stack. Goal feelings show rising neglect (no recent progress updates) and falling confidence. The daemon flags it as needing attention. Blocker prophecy predicts likely obstacles based on related goals. The agent receives early warning and can either unblock the goal, re-prioritize it, or honestly abandon it.

## Honest Limitations

AgenticPlanning does **not**:

- **Execute tasks** — It tracks goals, decisions, and commitments. It does not write code, send emails, or perform actions.
- **Replace project management tools** — It's not Jira, Linear, or Asana. It doesn't have boards, sprints, or team dashboards. It's a structured memory for AI agent planning.
- **Guarantee outcomes** — Physics and feelings model dynamics, they don't control them. A goal with high momentum can still fail. A well-crystallized decision can still be wrong.
- **Scale to thousands of goals** — Designed for agent-scale planning (5-100 goals). A 100-goal project at ~300 KB is the practical upper bound for responsive single-file operation.

**When it's overkill:**

- Single-session tasks with no need for continuity
- Trivial decisions with no alternatives worth preserving
- Solo agents working on isolated, independent tasks with no commitment tracking needs
- Projects where the entire context fits comfortably in an LLM's context window
