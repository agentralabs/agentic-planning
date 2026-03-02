---
status: stable
---

# Overview

AgenticPlanning is a persistent planning engine that gives AI agents living goals, crystallized decisions, and tracked commitments — all stored in a portable `.aplan` file.

## The Problem

AI agents suffer from four fundamental planning failures:

- **Goal drift** — Intentions stated in session 1 are forgotten by session 3. Without persistence, agents restart from zero every conversation.
- **Decision amnesia** — Agents make choices but lose the reasoning, alternatives, and trade-offs. When context changes, they can't revisit or explain past decisions.
- **Commitment overload** — Agents accumulate promises without tracking capacity, stakeholder importance, or conflicts between obligations.
- **Progress blindness** — No visibility into whether goals are advancing, stalled, or sliding backward. No early warning system for trouble.

## Core Capabilities

### Living Goals

Goals are not static checkboxes. Each goal has:

- **Physics** — momentum (rate of progress change), gravity (resource attraction), inertia (resistance to starting), energy (available force)
- **Feelings** — urgency (deadline pressure), neglect (time since attention), confidence (success likelihood), alignment (singularity fit), vitality (overall health)
- **Lifecycle** — Draft, Active, Blocked, Paused, Completed, Abandoned, Superseded, Reborn

### Decision Crystallization

Decisions follow a deliberation lifecycle: Pending, Deliberating, Crystallized, Regretted, Recrystallized. When a decision crystallizes, unchosen alternatives are preserved as **shadow paths** — available for resurrection if context changes. Full causal chains track which decisions led to which outcomes.

### Commitment Entanglement

Commitments model promises with stakeholder importance, due dates, and breaking costs. **Entanglement types** (Sequential, Parallel, Inverse, Resonant, Dependent) detect conflicts and coordination requirements between obligations.

### Dream Forecasting

The dream system generates completion scenarios, predicts obstacles, extracts actionable insights, and discovers sub-goals. Dreams are forward-looking simulations grounded in current goal state.

### Intention Singularity

The convergence point where all goals, decisions, and commitments align into a unified vision. Singularity collapse computes goal positions, identifies themes and tension lines, and reveals the **golden path** — the optimal execution sequence.

## The .aplan Artifact

All planning state lives in a single `.aplan` file — a portable binary format with checksums and crash recovery. It's the planning analog of `.amem` for memory.

An `.aplan` file contains: goals with physics and feelings, decisions with shadow paths, commitments with entanglements, dreams, federations, soul archives, and indexes. Copy it to another machine and planning state travels with it.

## Five Surfaces

One engine, many interfaces:

| Surface | Use Case |
|---------|----------|
| **MCP Server** | 13 tools for Claude Code and AI agent integration |
| **CLI** (`aplan`) | Interactive terminal workflows and scripting |
| **Rust API** | Direct library usage in Rust applications |
| **C FFI** | 27 exported functions for Python (ctypes), C, and other languages |
| **WASM/npm** | `@agenticamem/planning` for Node.js and browser use |

## Sister Ecosystem

AgenticPlanning is one of eight sister packages in the Agentra ecosystem:

| Sister | Integration |
|--------|------------|
| **agentic-memory** | Persist goals as memories, recall past planning context |
| **agentic-identity** | Sign decisions with cryptographic receipts |
| **agentic-time** | Deadline management, temporal scheduling |
| **agentic-contract** | Commitment governance, policy enforcement |
| **agentic-vision** | Visual goal tracking, UI state awareness |
| **agentic-codebase** | Link goals to code symbols and architecture |
| **agentic-comm** | Multi-agent planning coordination channels |

Bridges connect planning to each sister. In standalone mode, no-op bridges keep the engine self-contained.

## Who Should Use This

- **AI agent builders** who need persistent intention tracking across sessions
- **Multi-agent coordinators** managing shared goals through federation
- **Long-running workflow systems** requiring decision audit trails and commitment tracking

## Quick Example

```bash
# Create a goal with intention and priority
aplan goal create "Ship v1.0 release" --intention "Deliver production-ready planning" --priority high --activate

# Track progress
aplan goal progress <ID> 30 --note "Core engine complete"

# Check how the goal feels
aplan goal feelings <ID>
# → urgency: 0.65  neglect: 0.12  confidence: 0.71  vitality: 0.68

# Make a decision
aplan decision create --question "Which serialization format?" --context "Need speed and portability"

# Collapse the singularity to see the unified vision
aplan singularity collapse
```
