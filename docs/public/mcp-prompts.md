---
status: stable
---

# MCP Prompts

AgenticPlanning provides 4 prompt templates for common planning workflows.

## planning_review

Generate a planning review for a given time period.

**Arguments:**

| Name | Type | Description |
|------|------|-------------|
| `period` | string | Review period: `daily`, `weekly`, or `monthly` |

**Template:** Produces a structured review covering:
- Goals activated, completed, and abandoned during the period
- Progress velocity and momentum changes
- Decisions crystallized
- Commitments fulfilled, broken, or renegotiated
- Blocker analysis
- Singularity drift assessment
- Recommendations for the next period

---

## goal_decomposition

Decompose a goal into actionable sub-goals.

**Arguments:**

| Name | Type | Description |
|------|------|-------------|
| `goal_id` | string | UUID of the goal to decompose |

**Template:** Analyzes the goal's soul (intention, significance, success criteria) and produces:
- Recommended sub-goals with titles and descriptions
- Dependency ordering between sub-goals
- Estimated effort and risk per sub-goal
- Critical path identification
- Suggested deadlines relative to parent deadline

---

## decision_analysis

Analyze a pending decision with structured reasoning.

**Arguments:**

| Name | Type | Description |
|------|------|-------------|
| `question` | string | The decision question to analyze |

**Template:** Generates a decision analysis framework including:
- Restatement of the question with identified constraints
- Option generation (at least 3 paths)
- Pros/cons for each option
- Risk assessment
- Reversibility evaluation
- Recommended path with reasoning
- Shadow path documentation

---

## commitment_check

Assess commitment health across stakeholders.

**Arguments:**

| Name | Type | Description |
|------|------|-------------|
| `stakeholder` | string | Optional stakeholder name to focus on (omit for all) |

**Template:** Produces a health report covering:
- Active commitment count and total weight
- Overload assessment
- At-risk commitments with reasons
- Commitments due soon
- Entanglement analysis (coupled commitments)
- Trust impact projections
- Renegotiation recommendations
