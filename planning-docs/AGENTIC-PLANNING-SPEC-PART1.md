# AGENTIC PLANNING SPECIFICATION — PART 1

> **Specs Covered:** SPEC-01 through SPEC-04
> **Sister:** #8 of 25
> **Format:** .aplan
> **Dependencies:** Time ✅ + Contract ✅

---

# SPEC-01: OVERVIEW & DEPENDENCIES

## 1.1 Problem Statement

AI agents suffer from **goal amnesia** — they forget what they're trying to achieve between sessions, lose track of progress, make decisions without recording reasoning, and cannot be held accountable for commitments. This makes them unreliable for any work that spans multiple sessions.

## 1.2 Solution

AgenticPlanning provides **persistent intention infrastructure**:

- **Living Goals** — Goals that persist, evolve, and have relationships
- **Crystallized Decisions** — Choices recorded with reasoning and alternatives
- **Weighted Commitments** — Promises with accountability
- **Progress Physics** — Momentum, gravity, and blockers
- **Cross-Session Continuity** — Work survives restarts, crashes, years

## 1.3 Core Principles

```
1. GOALS ARE LIVING ENTITIES
   They are born, grow, struggle, transform, and die.
   They have feelings (urgency, neglect, confidence).
   They have relationships (dependencies, alliances, rivalries).

2. DECISIONS CRYSTALLIZE REALITY
   Before: infinite possibilities exist.
   After: one path is real, others are shadows.
   Shadows are preserved for counterfactual analysis.

3. COMMITMENTS HAVE WEIGHT
   Promises affect trajectory.
   Breaking them costs energy.
   Fulfilling them releases energy.

4. PROGRESS HAS PHYSICS
   Momentum resists stopping.
   Gravity attracts resources.
   Blockers can be prophesied.
   Completion sends echoes backward.

5. PLANNING SPANS TIME
   Goals survive sessions, restarts, years.
   Decisions are traceable forever.
   The agent can explain "why" for any state.
```

## 1.4 Dependencies

### Required Sisters

| Sister | Version | Purpose |
|--------|---------|---------|
| Temporal Bridge | ≥0.1.0 | Deadlines, scheduling, temporal reasoning |
| AgenticContract | ≥0.1.0 | Commitment enforcement, policy boundaries |

### Integration Sisters

| Sister | Purpose |
|--------|---------|
| AgenticMemory | Goals persist as memories, context retrieval |
| AgenticIdentity | Receipt signing, accountability chain |
| AgenticCognition | User modeling for goal prioritization |

### Rust Dependencies

```toml
[dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
blake3 = "1.5"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Async
tokio = { version = "1.0", features = ["full"] }

# Graph
petgraph = "0.6"

# CLI
clap = { version = "4.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Sister dependencies
temporal-bridge = "0.1"
agentic-contract = "0.1"
```

## 1.5 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           AGENTIC PLANNING ARCHITECTURE                          │
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                              MCP SERVER                                     │ │
│  │   12 consolidated tools × ~10 operations each = 120 operations             │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                           │
│                                      ▼                                           │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                           PLANNING ENGINE                                   │ │
│  │                                                                             │ │
│  │   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐         │ │
│  │   │    Goal     │ │  Decision   │ │ Commitment  │ │  Progress   │         │ │
│  │   │   Manager   │ │  Crystals   │ │   Ledger    │ │   Physics   │         │ │
│  │   └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘         │ │
│  │                                                                             │ │
│  │   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐         │ │
│  │   │  Intention  │ │    Dream    │ │  Prophecy   │ │ Federation  │         │ │
│  │   │ Singularity │ │   Engine    │ │   Engine    │ │   Manager   │         │ │
│  │   └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘         │ │
│  │                                                                             │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                           │
│                                      ▼                                           │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                           STORAGE LAYER                                     │ │
│  │                                                                             │ │
│  │   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐         │ │
│  │   │    Goal     │ │  Decision   │ │ Commitment  │ │    Dream    │         │ │
│  │   │    Graph    │ │   Crystal   │ │   Ledger    │ │   Archive   │         │ │
│  │   │   Store     │ │    Store    │ │    Store    │ │    Store    │         │ │
│  │   └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘         │ │
│  │                                                                             │ │
│  │   ┌─────────────────────────────────────────────────────────────┐         │ │
│  │   │                    INDEXES                                   │         │ │
│  │   │  Goal Index │ Decision Index │ Commitment Index │ Time Index │         │ │
│  │   └─────────────────────────────────────────────────────────────┘         │ │
│  │                                                                             │ │
│  │                          .aplan file                                       │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

# SPEC-02: CORE CONCEPTS

## 2.1 The Six Entities

AgenticPlanning manages six fundamental entity types:

```
ENTITIES:
═════════

1. GOAL         — A living intention with lifecycle
2. DECISION     — A crystallized choice with shadows
3. COMMITMENT   — A weighted promise with accountability
4. PROGRESS     — Physical properties of advancement
5. DREAM        — A simulated future state
6. FEDERATION   — Cross-agent goal coordination
```

## 2.2 Goal Lifecycle

```
GOAL LIFECYCLE:
═══════════════

                    ┌──────────────┐
                    │    DRAFT     │
                    │  (defining)  │
                    └──────┬───────┘
                           │ activate
                           ▼
                    ┌──────────────┐
       ┌───────────▶│    ACTIVE    │◀───────────┐
       │            │ (in progress)│            │
       │            └──────┬───────┘            │
       │                   │                    │
       │      ┌────────────┼────────────┐       │
       │      │            │            │       │
       │      ▼            ▼            ▼       │
       │ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
       │ │ BLOCKED │ │ PAUSED  │ │COMPLETED│   │
       │ │(waiting)│ │ (user)  │ │ (done)  │   │
       │ └────┬────┘ └────┬────┘ └─────────┘   │
       │      │           │                     │
       │      │ unblock   │ resume              │
       └──────┴───────────┴─────────────────────┘
       
       │
       │ abandon / supersede
       ▼
  ┌─────────────┐
  │    DEAD     │
  │ (archived)  │
  └─────────────┘
       │
       │ reincarnate (with new context)
       ▼
  ┌─────────────┐
  │  REBORN     │
  │ (new life)  │
  └─────────────┘
```

## 2.3 Decision States

```
DECISION STATES:
════════════════

  PENDING          → Question posed, not yet answered
  DELIBERATING     → Options being evaluated
  CRYSTALLIZED     → Choice made, reality collapsed
  REGRETTED        → Wishing different path taken
  RECRYSTALLIZED   → Decision changed (expensive)
```

## 2.4 Commitment States

```
COMMITMENT STATES:
══════════════════

  ACTIVE           → Promise outstanding
  AT_RISK          → May not be fulfilled
  RENEGOTIATING    → Terms being changed
  FULFILLED        → Promise kept
  BROKEN           → Promise broken
  RELEASED         → Other party released us
```

## 2.5 Relationships Between Entities

```
ENTITY RELATIONSHIPS:
═════════════════════

GOAL ──────────────────────────────────────────────────────────────
  │
  ├── has many → DECISIONS (choices made for this goal)
  ├── has many → COMMITMENTS (promises made about this goal)
  ├── has many → DREAMS (simulated completions)
  ├── has many → PROGRESS records
  │
  ├── parent of → GOALS (sub-goals)
  ├── depends on → GOALS (must complete first)
  ├── allied with → GOALS (synergistic)
  ├── rivals with → GOALS (competing)
  ├── entangled with → GOALS (romantic coupling)
  │
  └── belongs to → FEDERATION (if multi-agent)

DECISION ──────────────────────────────────────────────────────────
  │
  ├── belongs to → GOAL
  ├── has chosen → PATH (the reality)
  ├── has shadows → PATHS[] (unchosen alternatives)
  ├── caused by → DECISION (chain link)
  └── causes → DECISIONS[] (downstream effects)

COMMITMENT ─────────────────────────────────────────────────────────
  │
  ├── belongs to → GOAL
  ├── made to → STAKEHOLDER
  ├── entangled with → COMMITMENTS[]
  └── produces → FULFILLMENT (when completed)
```

## 2.6 Goal Feelings

Goals have feelings that affect their behavior:

```rust
pub struct GoalFeelings {
    /// How urgent is completion? (deadline pressure)
    /// 0.0 = no pressure, 1.0 = critical
    pub urgency: f64,
    
    /// How neglected is this goal?
    /// 0.0 = recently worked on, 1.0 = abandoned
    pub neglect: f64,
    
    /// How confident in completion?
    /// 0.0 = hopeless, 1.0 = certain
    pub confidence: f64,
    
    /// How aligned with original intention?
    /// 0.0 = completely drifted, 1.0 = perfectly on track
    pub alignment: f64,
    
    /// How alive/energetic is this goal?
    /// 0.0 = dying, 1.0 = thriving
    pub vitality: f64,
}
```

**Feeling Calculations:**

```
urgency = max(0, 1 - (days_until_deadline / deadline_buffer))
          + 0.2 * dependency_pressure
          + 0.1 * stakeholder_pressure

neglect = days_since_last_progress / neglect_threshold
          * (1 - progress_percentage)

confidence = progress_percentage
             * (1 - blocker_severity)
             * resource_availability
             * historical_success_rate

alignment = cosine_similarity(current_trajectory, original_intention)

vitality = (1 - neglect)
           * confidence
           * (1 - blocker_count * 0.1)
           * momentum
```

## 2.7 Progress Physics

Progress has physical properties:

```
MOMENTUM:
─────────
m(t) = m(t-1) × decay + progress(t) × energy

Where:
  decay = 0.95 per day (momentum naturally fades)
  progress = measurable advancement (0-1)
  energy = effort applied (0-1)

High momentum (>0.7): Hard to stop, resistant to abandonment
Low momentum (<0.3): Easy to drift, at risk of being forgotten


GRAVITY:
────────
g = importance × urgency × investment × emotional_weight

High gravity (>0.7): Attracts resources, attention, related goals
Low gravity (<0.3): Struggles to attract anything, drifts in space


INERTIA:
────────
i = scope × complexity × dependencies

High inertia: Hard to change direction
Low inertia: Easily pivots
```

## 2.8 Decision Crystallization

When a decision crystallizes:

```
BEFORE CRYSTALLIZATION:
═══════════════════════

  ┌─────────────────────────────────────────┐
  │          POSSIBILITY SPACE              │
  │                                         │
  │   Path A ─────────────▶ Future A       │
  │   Path B ─────────────▶ Future B       │
  │   Path C ─────────────▶ Future C       │
  │                                         │
  │   All paths equally real (superposition)│
  └─────────────────────────────────────────┘


THE MOMENT OF CRYSTALLIZATION:
══════════════════════════════

  User/Agent chooses Path B
  
  ╔═════════════════════════════════════════╗
  ║           REALITY COLLAPSE              ║
  ║                                         ║
  ║   Path A ░░░░░░░░░░░▶ [shadow]         ║
  ║   Path B ████████████▶ REALITY         ║
  ║   Path C ░░░░░░░░░░░▶ [shadow]         ║
  ║                                         ║
  ╚═════════════════════════════════════════╝


AFTER CRYSTALLIZATION:
══════════════════════

  Path B is the timeline.
  Paths A and C are preserved as crystal shadows.
  
  Shadows can be:
    • Consulted (counterfactual analysis)
    • Regretted (if Path B goes poorly)
    • Recrystallized (expensive, cascading changes)
```

---

# SPEC-03: DATA STRUCTURES

## 3.1 Core Types

```rust
//! Core type definitions for AgenticPlanning

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// IDENTIFIERS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoalId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitmentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DreamId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FederationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StakeholderId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathId(pub Uuid);

// ============================================================================
// TIMESTAMP
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now().timestamp_nanos_opt().unwrap_or(0))
    }
    
    pub fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }
    
    pub fn as_nanos(&self) -> i64 {
        self.0
    }
}

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    Draft,
    Active,
    Blocked,
    Paused,
    Completed,
    Abandoned,
    Superseded,
    Reborn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical,   // Must complete, everything else stops
    High,       // Important, gets resources first
    Medium,     // Normal priority
    Low,        // Nice to have
    Someday,    // No timeline
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionStatus {
    Pending,
    Deliberating,
    Crystallized,
    Regretted,
    Recrystallized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentStatus {
    Active,
    AtRisk,
    Renegotiating,
    Fulfilled,
    Broken,
    Released,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalRelationship {
    ParentChild { parent: GoalId, child: GoalId },
    Dependency { dependent: GoalId, on: GoalId, strength: f64 },
    Alliance { goals: (GoalId, GoalId), synergy: f64 },
    Rivalry { goals: (GoalId, GoalId), contested: Vec<String> },
    Romance { goals: (GoalId, GoalId), emergent_value: String },
    Nemesis { goals: (GoalId, GoalId), reason: String },
    Successor { predecessor: GoalId, successor: GoalId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockerType {
    ResourceUnavailable { resource: String },
    DependencyBlocked { goal: GoalId },
    DeadlineMiss { deadline: Timestamp },
    ExternalEvent { event: String },
    SkillGap { skill: String },
    ApprovalPending { approver: StakeholderId },
    TechnicalDebt { description: String },
    Unknown { signals: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CausalityType {
    Enables,
    Constrains,
    Suggests,
    Requires,
    Precludes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntanglementType {
    Sequential,
    Parallel,
    Inverse,
    Resonant,
    Dependent,
}
```

## 3.2 Goal Structures

```rust
// ============================================================================
// GOAL
// ============================================================================

/// A living goal with full consciousness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    // Identity
    pub id: GoalId,
    pub title: String,
    pub description: String,
    
    // Soul
    pub soul: GoalSoul,
    
    // Lifecycle
    pub status: GoalStatus,
    pub created_at: Timestamp,
    pub activated_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
    
    // Hierarchy
    pub parent: Option<GoalId>,
    pub children: Vec<GoalId>,
    
    // Relationships
    pub dependencies: Vec<GoalId>,
    pub dependents: Vec<GoalId>,
    pub relationships: Vec<GoalRelationship>,
    
    // State
    pub priority: Priority,
    pub progress: Progress,
    pub feelings: GoalFeelings,
    pub physics: GoalPhysics,
    
    // Blockers
    pub blockers: Vec<Blocker>,
    
    // Linked entities
    pub decisions: Vec<DecisionId>,
    pub commitments: Vec<CommitmentId>,
    pub dreams: Vec<DreamId>,
    
    // Metadata
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    
    // Provenance
    pub provenance: GoalProvenance,
    
    // Evolution
    pub metamorphosis: Option<GoalMetamorphosis>,
    pub previous_life: Option<PreviousLife>,
}

/// The essential nature of a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSoul {
    /// The original intention in user's words
    pub intention: String,
    
    /// Why this goal matters
    pub significance: String,
    
    /// What completion looks like
    pub success_criteria: Vec<SuccessCriterion>,
    
    /// User's emotional investment (0-1)
    pub emotional_weight: f64,
    
    /// Connection to user's values
    pub values: Vec<String>,
}

/// A success criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub id: Uuid,
    pub description: String,
    pub measurable: bool,
    pub metric: Option<String>,
    pub target: Option<f64>,
    pub achieved: bool,
    pub achieved_at: Option<Timestamp>,
}

/// Progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Overall completion (0-1)
    pub percentage: f64,
    
    /// Progress history
    pub history: Vec<ProgressPoint>,
    
    /// Velocity (progress per day)
    pub velocity: f64,
    
    /// Estimated completion
    pub eta: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPoint {
    pub timestamp: Timestamp,
    pub percentage: f64,
    pub note: Option<String>,
}

/// Goal feelings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalFeelings {
    pub urgency: f64,
    pub neglect: f64,
    pub confidence: f64,
    pub alignment: f64,
    pub vitality: f64,
    pub last_calculated: Timestamp,
}

/// Goal physics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPhysics {
    pub momentum: f64,
    pub gravity: f64,
    pub inertia: f64,
    pub energy: f64,
    pub last_calculated: Timestamp,
}

/// A blocker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    pub id: Uuid,
    pub blocker_type: BlockerType,
    pub description: String,
    pub severity: f64,
    pub identified_at: Timestamp,
    pub resolved_at: Option<Timestamp>,
    pub resolution: Option<String>,
}

/// Goal provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProvenance {
    /// How the goal was created
    pub origin: ProvenanceOrigin,
    
    /// Original user request
    pub user_request: Option<String>,
    
    /// Session where created
    pub session_id: Option<String>,
    
    /// Context at creation
    pub creation_context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvenanceOrigin {
    UserRequest,
    Decomposition { parent: GoalId },
    Reincarnation { previous: GoalId },
    Dream { dream: DreamId },
    Federation { federation: FederationId },
    System,
}

/// Goal metamorphosis tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMetamorphosis {
    pub stages: Vec<MetamorphicStage>,
    pub current_stage: usize,
    pub invariant_soul: GoalSoul,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetamorphicStage {
    pub stage_number: usize,
    pub title: String,
    pub description: String,
    pub entered_at: Timestamp,
    pub scope_change: ScopeChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeChange {
    Expansion { factor: f64, reason: String },
    Contraction { factor: f64, reason: String },
    Pivot { new_direction: String, reason: String },
    Refinement { clarification: String },
}

/// Previous life for reincarnated goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousLife {
    pub original_id: GoalId,
    pub death_cause: String,
    pub lessons_learned: Vec<String>,
    pub karma: GoalKarma,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalKarma {
    pub failures: Vec<String>,
    pub near_successes: Vec<String>,
    pub requirements_for_success: Vec<String>,
    pub invested_energy: f64,
}
```

## 3.3 Decision Structures

```rust
// ============================================================================
// DECISION
// ============================================================================

/// A crystallized decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    
    /// The question being answered
    pub question: DecisionQuestion,
    
    /// Status
    pub status: DecisionStatus,
    
    /// When crystallized
    pub crystallized_at: Option<Timestamp>,
    
    /// The chosen path
    pub chosen: Option<DecisionPath>,
    
    /// Unchosen paths (shadows)
    pub shadows: Vec<CrystalShadow>,
    
    /// Reasoning
    pub reasoning: DecisionReasoning,
    
    /// Who decided
    pub decider: Decider,
    
    /// Affected goals
    pub affected_goals: Vec<GoalId>,
    
    /// Causality
    pub caused_by: Option<DecisionId>,
    pub causes: Vec<DecisionId>,
    
    /// Reversibility
    pub reversibility: Reversibility,
    
    /// Observed consequences
    pub consequences: Vec<Consequence>,
    
    /// Regret tracking
    pub regret_score: f64,
    pub regret_updated_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionQuestion {
    pub question: String,
    pub context: String,
    pub constraints: Vec<String>,
    pub asked_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPath {
    pub id: PathId,
    pub name: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_effort: Option<f64>,
    pub estimated_risk: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalShadow {
    pub path: DecisionPath,
    pub rejection_reason: String,
    pub counterfactual: Option<CounterfactualProjection>,
    pub resurrection_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualProjection {
    pub projected_at: Timestamp,
    pub timeline: Vec<ProjectedEvent>,
    pub final_state: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedEvent {
    pub time_offset_days: f64,
    pub event: String,
    pub probability: f64,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReasoning {
    pub rationale: String,
    pub factors_considered: Vec<String>,
    pub weights: HashMap<String, f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decider {
    User { name: Option<String> },
    Agent { agent_id: String },
    Consensus { participants: Vec<StakeholderId> },
    Delegation { from: StakeholderId, to: StakeholderId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reversibility {
    pub is_reversible: bool,
    pub reversal_cost: f64,
    pub reversal_window: Option<Timestamp>,
    pub cascade_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consequence {
    pub observed_at: Timestamp,
    pub description: String,
    pub was_predicted: bool,
    pub impact: Impact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Impact {
    Positive,
    Negative,
    Neutral,
    Mixed,
}
```

## 3.4 Commitment Structures

```rust
// ============================================================================
// COMMITMENT
// ============================================================================

/// A weighted commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub id: CommitmentId,
    
    /// What was promised
    pub promise: Promise,
    
    /// Who it was made to
    pub made_to: Stakeholder,
    
    /// When made
    pub made_at: Timestamp,
    
    /// When due
    pub due: Option<Timestamp>,
    
    /// Status
    pub status: CommitmentStatus,
    
    /// Physics
    pub weight: f64,
    pub inertia: f64,
    
    /// Breaking cost
    pub breaking_cost: BreakingCost,
    
    /// Associated goal
    pub goal: Option<GoalId>,
    
    /// Entanglements
    pub entanglements: Vec<CommitmentEntanglement>,
    
    /// Fulfillment record
    pub fulfillment: Option<CommitmentFulfillment>,
    
    /// Renegotiation history
    pub renegotiations: Vec<Renegotiation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promise {
    pub description: String,
    pub deliverables: Vec<String>,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stakeholder {
    pub id: StakeholderId,
    pub name: String,
    pub role: String,
    pub importance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingCost {
    pub trust_damage: f64,
    pub relationship_impact: f64,
    pub reputation_cost: f64,
    pub energy_to_break: f64,
    pub cascading_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentEntanglement {
    pub with: CommitmentId,
    pub entanglement_type: EntanglementType,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentFulfillment {
    pub fulfilled_at: Timestamp,
    pub how_delivered: String,
    pub energy_released: f64,
    pub trust_gained: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Renegotiation {
    pub renegotiated_at: Timestamp,
    pub original: Promise,
    pub new: Promise,
    pub reason: String,
    pub accepted: bool,
    pub trust_impact: f64,
}
```

## 3.5 Dream Structures

```rust
// ============================================================================
// DREAM
// ============================================================================

/// A goal's dream of completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dream {
    pub id: DreamId,
    pub goal_id: GoalId,
    pub dreamt_at: Timestamp,
    
    /// The completion scenario
    pub scenario: CompletionScenario,
    
    /// Obstacles seen in dream
    pub obstacles: Vec<DreamObstacle>,
    
    /// Insights gained
    pub insights: Vec<DreamInsight>,
    
    /// Sub-goals discovered
    pub discovered_goals: Vec<GoalSeed>,
    
    /// Prophetic confidence
    pub confidence: f64,
    
    /// Was this dream accurate?
    pub accuracy: Option<DreamAccuracy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionScenario {
    pub vision: String,
    pub feeling: String,
    pub world_changes: Vec<String>,
    pub stakeholder_reactions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamObstacle {
    pub description: String,
    pub severity: f64,
    pub timing: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamInsight {
    pub insight: String,
    pub actionable: bool,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSeed {
    pub title: String,
    pub description: String,
    pub parent: GoalId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamAccuracy {
    pub assessed_at: Timestamp,
    pub accuracy_score: f64,
    pub correct_predictions: Vec<String>,
    pub incorrect_predictions: Vec<String>,
}
```

## 3.6 Federation Structures

```rust
// ============================================================================
// FEDERATION
// ============================================================================

/// A federated goal across agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub id: FederationId,
    pub goal_id: GoalId,
    pub created_at: Timestamp,
    
    /// Members
    pub members: Vec<FederationMember>,
    
    /// Coordinator
    pub coordinator: Option<String>,
    
    /// Sync state
    pub last_sync: Timestamp,
    pub sync_status: SyncStatus,
    
    /// Collective dreams
    pub collective_dreams: Vec<CollectiveDream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMember {
    pub agent_id: String,
    pub joined_at: Timestamp,
    pub owned_goals: Vec<GoalId>,
    pub progress: f64,
    pub status: MemberStatus,
    pub last_active: Timestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MemberStatus {
    Active,
    Inactive,
    Blocked,
    Left,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Pending,
    Conflict,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveDream {
    pub id: DreamId,
    pub participants: Vec<String>,
    pub individual_dreams: HashMap<String, Dream>,
    pub synthesis: DreamSynthesis,
    pub coherence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamSynthesis {
    pub unified_vision: String,
    pub themes: Vec<String>,
    pub conflicts: Vec<String>,
    pub resolutions: Vec<String>,
    pub emergent_goals: Vec<GoalSeed>,
}
```

---

# SPEC-04: FILE FORMAT

## 4.1 Overview

The `.aplan` file format stores the complete planning state:

```
APLAN FILE STRUCTURE:
═════════════════════

┌──────────────────────────────────────────────────────────────────┐
│ HEADER (128 bytes)                                               │
├──────────────────────────────────────────────────────────────────┤
│ SECTION: GOAL GRAPH                                              │
├──────────────────────────────────────────────────────────────────┤
│ SECTION: DECISION CRYSTALS                                       │
├──────────────────────────────────────────────────────────────────┤
│ SECTION: COMMITMENT LEDGER                                       │
├──────────────────────────────────────────────────────────────────┤
│ SECTION: DREAM ARCHIVE                                           │
├──────────────────────────────────────────────────────────────────┤
│ SECTION: FEDERATION STATE                                        │
├──────────────────────────────────────────────────────────────────┤
│ SECTION: INDEXES                                                 │
├──────────────────────────────────────────────────────────────────┤
│ FOOTER (64 bytes)                                                │
└──────────────────────────────────────────────────────────────────┘
```

## 4.2 Header Format

```rust
/// File header (128 bytes)
#[repr(C, packed)]
pub struct AplanHeader {
    /// Magic bytes: "PLAN"
    pub magic: [u8; 4],
    
    /// Format version
    pub version: u16,
    
    /// Flags
    pub flags: u32,
    
    /// Creation timestamp (nanos)
    pub created_at: i64,
    
    /// Last modified timestamp (nanos)
    pub modified_at: i64,
    
    /// Goal count
    pub goal_count: u32,
    
    /// Decision count
    pub decision_count: u32,
    
    /// Commitment count
    pub commitment_count: u32,
    
    /// Dream count
    pub dream_count: u32,
    
    /// Federation count
    pub federation_count: u32,
    
    /// Section offsets
    pub goal_section_offset: u64,
    pub decision_section_offset: u64,
    pub commitment_section_offset: u64,
    pub dream_section_offset: u64,
    pub federation_section_offset: u64,
    pub index_section_offset: u64,
    
    /// Checksum (Blake3 of all sections)
    pub checksum: [u8; 32],
    
    /// Reserved
    pub reserved: [u8; 14],
}

pub const APLAN_MAGIC: [u8; 4] = *b"PLAN";
pub const APLAN_VERSION: u16 = 1;
pub const APLAN_HEADER_SIZE: usize = 128;
```

## 4.3 Section Format

Each section has a standard structure:

```rust
/// Section header (32 bytes)
#[repr(C, packed)]
pub struct SectionHeader {
    /// Section type identifier
    pub section_type: u32,
    
    /// Section version
    pub version: u16,
    
    /// Flags
    pub flags: u16,
    
    /// Entry count
    pub entry_count: u32,
    
    /// Compressed size
    pub compressed_size: u64,
    
    /// Uncompressed size
    pub uncompressed_size: u64,
    
    /// Reserved
    pub reserved: [u8; 4],
}

/// Section types
pub const SECTION_GOALS: u32 = 1;
pub const SECTION_DECISIONS: u32 = 2;
pub const SECTION_COMMITMENTS: u32 = 3;
pub const SECTION_DREAMS: u32 = 4;
pub const SECTION_FEDERATIONS: u32 = 5;
pub const SECTION_INDEXES: u32 = 6;
```

## 4.4 Goal Section

```rust
/// Goal entry in file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEntry {
    pub goal: Goal,
    pub graph_edges: Vec<GoalEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEdge {
    pub from: GoalId,
    pub to: GoalId,
    pub edge_type: GoalEdgeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalEdgeType {
    Parent,
    Child,
    Dependency,
    Alliance,
    Rivalry,
    Romance,
    Nemesis,
}
```

## 4.5 Index Section

```rust
/// Index structures for fast lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanIndexes {
    /// Goals by status
    pub goals_by_status: HashMap<GoalStatus, Vec<GoalId>>,
    
    /// Goals by priority
    pub goals_by_priority: HashMap<Priority, Vec<GoalId>>,
    
    /// Goals by deadline (sorted)
    pub goals_by_deadline: Vec<(Timestamp, GoalId)>,
    
    /// Decisions by goal
    pub decisions_by_goal: HashMap<GoalId, Vec<DecisionId>>,
    
    /// Decisions by time
    pub decisions_by_time: Vec<(Timestamp, DecisionId)>,
    
    /// Commitments by due date
    pub commitments_by_due: Vec<(Timestamp, CommitmentId)>,
    
    /// Commitments by stakeholder
    pub commitments_by_stakeholder: HashMap<StakeholderId, Vec<CommitmentId>>,
    
    /// Active goals (quick access)
    pub active_goals: Vec<GoalId>,
    
    /// Blocked goals
    pub blocked_goals: Vec<GoalId>,
    
    /// Urgent items (deadline within 7 days)
    pub urgent: Vec<UrgentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrgentItem {
    pub item_type: UrgentItemType,
    pub id: Uuid,
    pub deadline: Timestamp,
    pub urgency: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UrgentItemType {
    Goal,
    Commitment,
    Decision,
}
```

## 4.6 Footer Format

```rust
/// File footer (64 bytes)
#[repr(C, packed)]
pub struct AplanFooter {
    /// Total file size
    pub file_size: u64,
    
    /// Number of write operations
    pub write_count: u64,
    
    /// Last session ID
    pub last_session: [u8; 16],
    
    /// Integrity marker
    pub integrity: [u8; 8],
    
    /// Footer checksum
    pub footer_checksum: [u8; 16],
    
    /// Reserved
    pub reserved: [u8; 8],
}

pub const APLAN_FOOTER_SIZE: usize = 64;
pub const APLAN_INTEGRITY_MARKER: [u8; 8] = *b"PLANEND\0";
```

## 4.7 File Operations

```rust
impl PlanningEngine {
    /// Open or create planning file
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        if path.exists() {
            Self::load(path)
        } else {
            Self::create(path)
        }
    }
    
    /// Create new planning file
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let header = AplanHeader::new();
        let mut file = File::create(path)?;
        
        // Write header
        file.write_all(&header.to_bytes())?;
        
        // Write empty sections
        // ...
        
        // Write footer
        let footer = AplanFooter::new(file.stream_position()?);
        file.write_all(&footer.to_bytes())?;
        
        Ok(Self::from_file(file))
    }
    
    /// Load existing planning file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        
        // Read and verify header
        let mut header_bytes = [0u8; APLAN_HEADER_SIZE];
        file.read_exact(&mut header_bytes)?;
        let header = AplanHeader::from_bytes(&header_bytes)?;
        
        if &header.magic != &APLAN_MAGIC {
            return Err(Error::InvalidMagic);
        }
        
        // Read sections
        // ...
        
        // Verify footer
        file.seek(SeekFrom::End(-(APLAN_FOOTER_SIZE as i64)))?;
        let mut footer_bytes = [0u8; APLAN_FOOTER_SIZE];
        file.read_exact(&mut footer_bytes)?;
        let footer = AplanFooter::from_bytes(&footer_bytes)?;
        
        if &footer.integrity != &APLAN_INTEGRITY_MARKER {
            return Err(Error::CorruptedFile);
        }
        
        Ok(Self::from_loaded(header, sections, footer))
    }
    
    /// Save planning state
    pub fn save(&mut self) -> Result<()> {
        // Atomic write: write to temp, then rename
        let temp_path = self.path.with_extension("aplan.tmp");
        
        // Write to temp
        self.write_to_file(&temp_path)?;
        
        // Atomic rename
        std::fs::rename(&temp_path, &self.path)?;
        
        Ok(())
    }
}
```

---

## Part 1 Complete

**Covered:**
- SPEC-01: Overview & Dependencies
- SPEC-02: Core Concepts
- SPEC-03: Data Structures
- SPEC-04: File Format

**Next (Part 2):**
- SPEC-05: Write Engine
- SPEC-06: Query Engine
- SPEC-07: Indexes
- SPEC-08: Validation

---

*Document: AGENTIC-PLANNING-SPEC-PART1.md*
