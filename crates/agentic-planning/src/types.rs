use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GoalId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DecisionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitmentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DreamId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FederationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StakeholderId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PathId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn days_from_now(days: f64) -> Self {
        Self(Self::now().0 + (days * 86_400.0 * 1e9) as i64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Someday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionStatus {
    Pending,
    Deliberating,
    Crystallized,
    Regretted,
    Recrystallized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    ParentChild {
        parent: GoalId,
        child: GoalId,
    },
    Dependency {
        dependent: GoalId,
        on: GoalId,
        strength: f64,
    },
    Alliance {
        goals: (GoalId, GoalId),
        synergy: f64,
    },
    Rivalry {
        goals: (GoalId, GoalId),
        contested: Vec<String>,
    },
    Romance {
        goals: (GoalId, GoalId),
        emergent_value: String,
    },
    Nemesis {
        goals: (GoalId, GoalId),
        reason: String,
    },
    Successor {
        predecessor: GoalId,
        successor: GoalId,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntanglementType {
    Sequential,
    Parallel,
    Inverse,
    Resonant,
    Dependent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub description: String,
    pub soul: GoalSoul,
    pub status: GoalStatus,
    pub created_at: Timestamp,
    pub activated_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
    pub parent: Option<GoalId>,
    pub children: Vec<GoalId>,
    pub dependencies: Vec<GoalId>,
    pub dependents: Vec<GoalId>,
    pub relationships: Vec<GoalRelationship>,
    pub priority: Priority,
    pub progress: Progress,
    pub feelings: GoalFeelings,
    pub physics: GoalPhysics,
    pub blockers: Vec<Blocker>,
    pub decisions: Vec<DecisionId>,
    pub commitments: Vec<CommitmentId>,
    pub dreams: Vec<DreamId>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub provenance: GoalProvenance,
    pub metamorphosis: Option<GoalMetamorphosis>,
    pub previous_life: Option<PreviousLife>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSoul {
    pub intention: String,
    pub significance: String,
    pub success_criteria: Vec<SuccessCriterion>,
    pub emotional_weight: f64,
    pub values: Vec<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub percentage: f64,
    pub history: Vec<ProgressPoint>,
    pub velocity: f64,
    pub eta: Option<Timestamp>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            percentage: 0.0,
            history: Vec::new(),
            velocity: 0.0,
            eta: None,
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPoint {
    pub timestamp: Timestamp,
    pub percentage: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalFeelings {
    pub urgency: f64,
    pub neglect: f64,
    pub confidence: f64,
    pub alignment: f64,
    pub vitality: f64,
    pub last_calculated: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPhysics {
    pub momentum: f64,
    pub gravity: f64,
    pub inertia: f64,
    pub energy: f64,
    pub last_calculated: Timestamp,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProvenance {
    pub origin: ProvenanceOrigin,
    pub user_request: Option<String>,
    pub session_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMetamorphosis {
    pub stages: Vec<MetamorphicStage>,
    pub current_stage: usize,
    pub invariant_soul: GoalSoul,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetamorphosisSignal {
    pub goal_id: GoalId,
    pub should_transform: bool,
    pub reason: String,
    pub recommended_change: ScopeChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetamorphosisPrediction {
    pub goal_id: GoalId,
    pub confidence: f64,
    pub next_change: ScopeChange,
    pub rationale: String,
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
    Expansion {
        factor: f64,
        reason: String,
    },
    Contraction {
        factor: f64,
        reason: String,
    },
    Pivot {
        new_direction: String,
        reason: String,
    },
    Refinement {
        clarification: String,
    },
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    pub question: DecisionQuestion,
    pub status: DecisionStatus,
    pub crystallized_at: Option<Timestamp>,
    pub chosen: Option<DecisionPath>,
    pub shadows: Vec<CrystalShadow>,
    pub reasoning: DecisionReasoning,
    pub decider: Decider,
    pub affected_goals: Vec<GoalId>,
    pub caused_by: Option<DecisionId>,
    pub causes: Vec<DecisionId>,
    pub reversibility: Reversibility,
    pub consequences: Vec<Consequence>,
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

impl Default for DecisionPath {
    fn default() -> Self {
        Self {
            id: PathId(Uuid::new_v4()),
            name: String::new(),
            description: String::new(),
            pros: Vec::new(),
            cons: Vec::new(),
            estimated_effort: None,
            estimated_risk: None,
        }
    }
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

impl Default for DecisionReasoning {
    fn default() -> Self {
        Self {
            rationale: String::new(),
            factors_considered: Vec::new(),
            weights: HashMap::new(),
            confidence: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decider {
    User {
        name: Option<String>,
    },
    Agent {
        agent_id: String,
    },
    Consensus {
        participants: Vec<StakeholderId>,
    },
    Delegation {
        from: StakeholderId,
        to: StakeholderId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reversibility {
    pub is_reversible: bool,
    pub reversal_cost: f64,
    pub reversal_window: Option<Timestamp>,
    pub cascade_count: usize,
}

impl Default for Reversibility {
    fn default() -> Self {
        Self {
            is_reversible: true,
            reversal_cost: 0.2,
            reversal_window: None,
            cascade_count: 0,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub id: CommitmentId,
    pub promise: Promise,
    pub made_to: Stakeholder,
    pub made_at: Timestamp,
    pub due: Option<Timestamp>,
    pub status: CommitmentStatus,
    pub weight: f64,
    pub inertia: f64,
    pub breaking_cost: BreakingCost,
    pub goal: Option<GoalId>,
    pub entanglements: Vec<CommitmentEntanglement>,
    pub fulfillment: Option<CommitmentFulfillment>,
    pub renegotiations: Vec<Renegotiation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Default for Stakeholder {
    fn default() -> Self {
        Self {
            id: StakeholderId(Uuid::new_v4()),
            name: String::new(),
            role: "stakeholder".to_string(),
            importance: 0.5,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dream {
    pub id: DreamId,
    pub goal_id: GoalId,
    pub dreamt_at: Timestamp,
    pub scenario: CompletionScenario,
    pub obstacles: Vec<DreamObstacle>,
    pub insights: Vec<DreamInsight>,
    pub discovered_goals: Vec<GoalSeed>,
    pub confidence: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub id: FederationId,
    pub goal_id: GoalId,
    pub created_at: Timestamp,
    pub members: Vec<FederationMember>,
    pub coordinator: Option<String>,
    pub last_sync: Timestamp,
    pub sync_status: SyncStatus,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTree {
    pub root: GoalId,
    pub nodes: HashMap<GoalId, GoalTreeNode>,
    pub edges: Vec<(GoalId, GoalId)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTreeNode {
    pub goal: Goal,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentionSingularity {
    pub unified_vision: String,
    pub goal_positions: HashMap<GoalId, IntentionPosition>,
    pub themes: Vec<String>,
    pub tension_lines: Vec<TensionLine>,
    pub golden_path: Vec<GoalId>,
    pub center: IntentionCenter,
}

impl Default for IntentionSingularity {
    fn default() -> Self {
        Self {
            unified_vision: String::new(),
            goal_positions: HashMap::new(),
            themes: Vec::new(),
            tension_lines: Vec::new(),
            golden_path: Vec::new(),
            center: IntentionCenter {
                urgency: 0.0,
                confidence: 0.0,
                momentum: 0.0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentionPosition {
    pub goal_id: GoalId,
    pub centrality: f64,
    pub alignment_angle: f64,
    pub gravitational_pull: f64,
    pub drift_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentionCenter {
    pub urgency: f64,
    pub confidence: f64,
    pub momentum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionLine {
    pub a: GoalId,
    pub b: GoalId,
    pub magnitude: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionChain {
    pub root: DecisionId,
    pub descendants: Vec<DecisionId>,
    pub causality: Vec<CausalLink>,
    pub cascade_analysis: CascadeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub from: DecisionId,
    pub to: DecisionId,
    pub causality_type: CausalityType,
    pub strength: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CascadeAnalysis {
    pub total_nodes: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionArchaeology {
    pub artifact: String,
    pub strata: Vec<ArchaeologicalStratum>,
    pub cumulative_impact: String,
    pub insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchaeologicalStratum {
    pub depth: usize,
    pub decision: DecisionId,
    pub age: String,
    pub impact_on_artifact: String,
    pub context_at_time: String,
    pub was_reasonable: bool,
    pub modern_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockerProphecy {
    pub goal_id: GoalId,
    pub predicted_blocker: Blocker,
    pub prediction_confidence: f64,
    pub days_until_materialization: f64,
    pub evidence: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEcho {
    pub goal_id: GoalId,
    pub source_milestone: Milestone,
    pub echo_strength: f64,
    pub estimated_arrival_secs: u64,
    pub carried_information: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionProphecy {
    pub question: DecisionQuestion,
    pub paths: Vec<ProphecyPath>,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProphecyPath {
    pub path: DecisionPath,
    pub timeline: Vec<ProjectedEvent>,
    pub final_state: String,
    pub risk_profile: String,
    pub opportunity_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentInventory {
    pub total_count: usize,
    pub active_count: usize,
    pub total_weight: f64,
    pub sustainable_weight: f64,
    pub is_overloaded: bool,
    pub by_stakeholder: HashMap<String, usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UrgentItemType {
    Goal,
    Commitment,
    Decision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrgentItem {
    pub item_type: UrgentItemType,
    pub id: Uuid,
    pub deadline: Timestamp,
    pub urgency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSoulArchive {
    pub original_id: GoalId,
    pub soul: GoalSoul,
    pub death_record: GoalDeath,
    pub karma: GoalKarma,
    pub reincarnation_potential: f64,
    pub trigger_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDeath {
    pub cause: String,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MergeReport {
    pub goals_merged: usize,
    pub decisions_merged: usize,
    pub commitments_merged: usize,
    pub dreams_merged: usize,
    pub federations_merged: usize,
    pub souls_merged: usize,
}

#[derive(Debug, Clone, Default)]
pub struct GoalFilter {
    pub status: Option<Vec<GoalStatus>>,
    pub priority: Option<Vec<Priority>>,
    pub parent: Option<GoalId>,
    pub has_deadline: Option<bool>,
    pub deadline_before: Option<Timestamp>,
    pub deadline_after: Option<Timestamp>,
    pub tags: Option<Vec<String>>,
    pub created_after: Option<Timestamp>,
    pub min_progress: Option<f64>,
    pub max_progress: Option<f64>,
    pub min_momentum: Option<f64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ReincarnationUpdates {
    pub title: Option<String>,
    pub description: Option<String>,
    pub lessons_learned: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateGoalRequest {
    pub title: String,
    pub description: String,
    pub intention: String,
    pub significance: Option<String>,
    pub success_criteria: Option<Vec<SuccessCriterion>>,
    pub emotional_weight: Option<f64>,
    pub values: Option<Vec<String>>,
    pub priority: Option<Priority>,
    pub deadline: Option<Timestamp>,
    pub parent: Option<GoalId>,
    pub dependencies: Option<Vec<GoalId>>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub origin: Option<ProvenanceOrigin>,
    pub user_request: Option<String>,
    pub session_id: Option<String>,
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateDecisionRequest {
    pub question: String,
    pub context: Option<String>,
    pub constraints: Option<Vec<String>>,
    pub goals: Option<Vec<GoalId>>,
    pub caused_by: Option<DecisionId>,
    pub decider: Option<Decider>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateCommitmentRequest {
    pub promise: Promise,
    pub stakeholder: Stakeholder,
    pub due: Option<Timestamp>,
    pub goal: Option<GoalId>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateGoalRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub deadline: Option<Option<Timestamp>>,
    pub priority: Option<Priority>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub intention: Option<String>,
    pub significance: Option<String>,
    pub emotional_weight: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateCommitmentRequest {
    pub promise: Option<Promise>,
    pub due: Option<Option<Timestamp>>,
    pub goal: Option<Option<GoalId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressForecast {
    pub goal_id: GoalId,
    pub current_percentage: f64,
    pub current_velocity: f64,
    pub projected_milestones: Vec<ForecastMilestone>,
    pub estimated_completion: Option<Timestamp>,
    pub confidence: f64,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastMilestone {
    pub percentage: f64,
    pub estimated_at: Timestamp,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumReport {
    pub total_goals: usize,
    pub average_momentum: f64,
    pub momentum_distribution: MomentumDistribution,
    pub top_momentum: Vec<GoalMomentumEntry>,
    pub stalled: Vec<GoalMomentumEntry>,
    pub accelerating: Vec<GoalMomentumEntry>,
    pub decelerating: Vec<GoalMomentumEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumDistribution {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub zero: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMomentumEntry {
    pub goal_id: GoalId,
    pub title: String,
    pub momentum: f64,
    pub velocity: f64,
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityField {
    pub total_goals: usize,
    pub field_center: GravityCenter,
    pub wells: Vec<GravityWell>,
    pub total_pull: f64,
    pub dominant_attractor: Option<GoalId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityCenter {
    pub weighted_urgency: f64,
    pub weighted_priority: f64,
    pub weighted_momentum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityWell {
    pub goal_id: GoalId,
    pub title: String,
    pub gravity: f64,
    pub pull_radius: f64,
    pub captured_goals: Vec<GoalId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationHealthEntry {
    pub federation_id: FederationId,
    pub goal_id: GoalId,
    pub member_count: usize,
    pub sync_age_hours: f64,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalHealthReport {
    pub total_active: usize,
    pub total_blocked: usize,
    pub stalled: Vec<GoalId>,
    pub neglected: Vec<GoalId>,
    pub deadline_risk: Vec<GoalId>,
    pub blocked: Vec<GoalId>,
    pub thriving: Vec<GoalId>,
}

// ═══════════════════════════════════════════════════════════════════
// Invention 12: Decision Consensus — Collective Wisdom
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusStatus {
    Open,
    Deliberating,
    Synthesizing,
    Voting,
    Crystallized,
    Deadlocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConsensus {
    pub decision_id: DecisionId,
    pub stakeholders: Vec<ConsensusParticipant>,
    pub deliberation: Vec<DeliberationRound>,
    pub synthesis: Option<Synthesis>,
    pub votes: HashMap<StakeholderId, String>,
    pub alignment_score: f64,
    pub status: ConsensusStatus,
    pub started_at: Timestamp,
    pub crystallized_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusParticipant {
    pub id: StakeholderId,
    pub role: String,
    pub initial_position: String,
    pub concerns: Vec<String>,
    pub requirements: Vec<String>,
    pub flexibility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationRound {
    pub round_number: usize,
    pub statements: Vec<ConsensusStatement>,
    pub alignment_delta: f64,
    pub emerged_common_ground: Vec<CommonGround>,
    pub recorded_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStatement {
    pub stakeholder_id: StakeholderId,
    pub position: String,
    pub supporting_arguments: Vec<String>,
    pub concessions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonGround {
    pub description: String,
    pub agreed_by: Vec<StakeholderId>,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    pub proposal: String,
    pub incorporates_from: Vec<StakeholderId>,
    pub addresses_concerns: Vec<String>,
    pub confidence: f64,
    pub proposed_at: Timestamp,
}
