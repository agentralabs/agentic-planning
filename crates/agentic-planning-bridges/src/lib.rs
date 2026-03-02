use agentic_planning::{
    BlockerProphecy, Commitment, CommitmentId, Decision, DecisionId, DecisionPath, Goal, GoalId,
    IntentionSingularity, PlanningEngine, Timestamp,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("bridge unavailable: {0}")]
    Unavailable(&'static str),
    #[error("policy denied by {policy}: {reason}")]
    PolicyDenied { policy: String, reason: String },
    #[error("invalid bridge input: {0}")]
    InvalidInput(String),
    #[error("planning core error: {0}")]
    Core(#[from] agentic_planning::Error),
}

pub type BridgeResult<T> = Result<T, BridgeError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Overdue,
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeContext {
    pub deadline: Timestamp,
    pub days_remaining: f64,
    pub is_overdue: bool,
    pub urgency_level: UrgencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    pub goal: Option<GoalId>,
    pub decision: Option<DecisionId>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyResult {
    Allowed,
    Denied { policy: String, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceResult {
    Compliant,
    Warning { reason: String },
    Violation { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    AutoApproved,
    Pending,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: MemoryId,
    pub summary: String,
    pub recorded_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub signature: String,
    pub signed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub verified_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    pub focus_areas: Vec<String>,
    pub risk_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub dominant_pattern: String,
    pub confidence: f64,
}

pub trait TimeBridge: Send + Sync {
    fn get_deadline_context(&self, deadline: Timestamp) -> TimeContext;
    fn schedule_check(&self, goal_id: GoalId, when: Timestamp) -> BridgeResult<()>;
    fn get_decay(&self, goal: &Goal) -> f64;
    fn calculate_time_urgency(&self, goal: &Goal) -> f64;
}

pub trait ContractBridge: Send + Sync {
    fn check_policy(&self, action: &str, context: &PolicyContext) -> PolicyResult;
    fn create_contract(&self, commitment: &Commitment) -> BridgeResult<ContractId>;
    fn check_compliance(&self, commitment_id: CommitmentId) -> ComplianceResult;
    fn request_approval(&self, decision: &Decision) -> BridgeResult<ApprovalStatus>;
}

pub trait MemoryBridge: Send + Sync {
    fn persist_goal(&self, goal: &Goal) -> BridgeResult<MemoryId>;
    fn get_goal_context(&self, goal_id: GoalId) -> Vec<MemoryRecord>;
    fn persist_decision(&self, decision: &Decision) -> BridgeResult<MemoryId>;
    fn search_context(&self, query: &str) -> Vec<MemoryRecord>;
}

pub trait IdentityBridge: Send + Sync {
    fn sign_decision(&self, decision: &Decision) -> BridgeResult<Receipt>;
    fn sign_commitment(&self, commitment: &Commitment) -> BridgeResult<Receipt>;
    fn verify_agent(&self, agent_id: &str) -> BridgeResult<AgentIdentity>;
    fn get_accountability_chain(&self, goal_id: GoalId) -> Vec<Receipt>;
}

pub trait CognitionBridge: Send + Sync {
    fn get_user_model(&self) -> UserModel;
    fn predict_preference(&self, options: &[DecisionPath]) -> Vec<f64>;
    fn analyze_patterns(&self, decisions: &[Decision]) -> PatternAnalysis;
}

pub trait VisionBridge: Send + Sync {
    fn capture_evidence(&self, goal_id: GoalId) -> BridgeResult<EvidenceId>;
    fn link_evidence(&self, goal_id: GoalId, evidence_id: EvidenceId) -> BridgeResult<()>;
}

/// Bridge to agentic-codebase for code-aware planning operations.
pub trait CodebaseBridge: Send + Sync {
    /// Link a goal to a code symbol for traceability
    fn link_goal_to_symbol(&self, goal_id: GoalId, symbol_name: &str) -> BridgeResult<()> {
        let _ = (goal_id, symbol_name);
        Err(BridgeError::Unavailable("codebase bridge not connected"))
    }

    /// Find code symbols related to a planning topic
    fn find_related_code(&self, topic: &str, max_results: usize) -> Vec<String> {
        let _ = (topic, max_results);
        Vec::new()
    }

    /// Get code context for enriching a goal or decision
    fn code_context(&self, symbol_name: &str) -> Option<String> {
        let _ = symbol_name;
        None
    }
}

/// Bridge to agentic-comm for message-linked planning operations.
pub trait CommBridge: Send + Sync {
    /// Broadcast a planning event to a comm channel
    fn broadcast_event(&self, event_type: &str, goal_id: GoalId) -> BridgeResult<()> {
        let _ = (event_type, goal_id);
        Err(BridgeError::Unavailable("comm bridge not connected"))
    }

    /// Notify stakeholders of a commitment change via comm
    fn notify_stakeholders(&self, commitment_id: CommitmentId, message: &str) -> BridgeResult<()> {
        let _ = (commitment_id, message);
        Err(BridgeError::Unavailable("comm bridge not connected"))
    }

    /// Store a planning discussion from a comm channel
    fn store_from_channel(&self, channel_id: u64, summary: &str) -> BridgeResult<MemoryId> {
        let _ = (channel_id, summary);
        Err(BridgeError::Unavailable("comm bridge not connected"))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdapterType {
    Sister,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    GoalManagement,
    DecisionSupport,
    CommitmentTracking,
    ProgressPhysics,
    IntentionSingularity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy {
        goals: usize,
        decisions: usize,
        commitments: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterRequest {
    GetActiveGoals,
    GetBlockers,
    GetSingularity,
    RecordProgress {
        goal_id: GoalId,
        percentage: f64,
        note: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterResponse {
    Goals(Vec<Goal>),
    Blockers(Vec<BlockerProphecy>),
    Singularity(Box<IntentionSingularity>),
    Goal(Box<Goal>),
}

pub trait HydraAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn adapter_type(&self) -> AdapterType;
    fn capabilities(&self) -> Vec<Capability>;
    fn health(&self) -> HealthStatus;
    fn handle(&mut self, request: AdapterRequest) -> BridgeResult<AdapterResponse>;
}

pub struct PlanningHydraAdapter {
    engine: PlanningEngine,
}

impl PlanningHydraAdapter {
    pub fn new(engine: PlanningEngine) -> Self {
        Self { engine }
    }

    pub fn engine(&self) -> &PlanningEngine {
        &self.engine
    }

    pub fn into_engine(self) -> PlanningEngine {
        self.engine
    }
}

impl HydraAdapter for PlanningHydraAdapter {
    fn id(&self) -> &str {
        "agentic-planning"
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Sister
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::GoalManagement,
            Capability::DecisionSupport,
            Capability::CommitmentTracking,
            Capability::ProgressPhysics,
            Capability::IntentionSingularity,
        ]
    }

    fn health(&self) -> HealthStatus {
        HealthStatus::Healthy {
            goals: self.engine.goal_count(),
            decisions: self.engine.decision_count(),
            commitments: self.engine.commitment_count(),
        }
    }

    fn handle(&mut self, request: AdapterRequest) -> BridgeResult<AdapterResponse> {
        match request {
            AdapterRequest::GetActiveGoals => Ok(AdapterResponse::Goals(
                self.engine
                    .get_active_goals()
                    .into_iter()
                    .cloned()
                    .collect(),
            )),
            AdapterRequest::GetBlockers => Ok(AdapterResponse::Blockers(
                self.engine.scan_blocker_prophecy(),
            )),
            AdapterRequest::GetSingularity => Ok(AdapterResponse::Singularity(Box::new(
                self.engine.get_intention_singularity(),
            ))),
            AdapterRequest::RecordProgress {
                goal_id,
                percentage,
                note,
            } => {
                let normalized = percentage.clamp(0.0, 1.0);
                let goal = self.engine.progress_goal(goal_id, normalized, note)?;
                Ok(AdapterResponse::Goal(Box::new(goal)))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TimeIntegration {
    scheduled_checks: Mutex<Vec<(GoalId, Timestamp)>>,
}

impl TimeIntegration {
    pub fn scheduled_checks(&self) -> Vec<(GoalId, Timestamp)> {
        self.scheduled_checks
            .lock()
            .expect("scheduled checks lock poisoned")
            .clone()
    }
}

impl TimeBridge for TimeIntegration {
    fn get_deadline_context(&self, deadline: Timestamp) -> TimeContext {
        NullTimeBridge.get_deadline_context(deadline)
    }

    fn schedule_check(&self, goal_id: GoalId, when: Timestamp) -> BridgeResult<()> {
        self.scheduled_checks
            .lock()
            .expect("scheduled checks lock poisoned")
            .push((goal_id, when));
        Ok(())
    }

    fn get_decay(&self, goal: &Goal) -> f64 {
        NullTimeBridge.get_decay(goal)
    }

    fn calculate_time_urgency(&self, goal: &Goal) -> f64 {
        NullTimeBridge.calculate_time_urgency(goal)
    }
}

#[derive(Debug)]
pub struct ContractIntegration {
    forbidden_tags: HashSet<String>,
    approval_threshold: f64,
    contracts: Mutex<HashMap<CommitmentId, ContractId>>,
}

impl Default for ContractIntegration {
    fn default() -> Self {
        Self {
            forbidden_tags: ["forbidden", "restricted", "blocked"]
                .iter()
                .map(|v| v.to_string())
                .collect(),
            approval_threshold: 0.7,
            contracts: Mutex::new(HashMap::new()),
        }
    }
}

impl ContractIntegration {
    pub fn with_forbidden_tags(tags: &[&str]) -> Self {
        Self {
            forbidden_tags: tags.iter().map(|v| v.to_string()).collect(),
            ..Default::default()
        }
    }

    fn assess_decision_risk(&self, decision: &Decision) -> f64 {
        let mut risk = 0.2;
        risk += (decision.shadows.len() as f64 * 0.08).min(0.3);
        let q = decision.question.question.to_ascii_lowercase();
        if q.contains("production") || q.contains("financial") || q.contains("legal") {
            risk += 0.35;
        }
        if !decision.question.constraints.is_empty() {
            risk += 0.15;
        }
        risk.clamp(0.0, 1.0)
    }
}

impl ContractBridge for ContractIntegration {
    fn check_policy(&self, action: &str, context: &PolicyContext) -> PolicyResult {
        if action.trim().is_empty() {
            return PolicyResult::Denied {
                policy: "action-non-empty".to_string(),
                reason: "action cannot be empty".to_string(),
            };
        }

        for tag in &context.tags {
            if self
                .forbidden_tags
                .contains(&tag.to_ascii_lowercase().to_string())
            {
                return PolicyResult::Denied {
                    policy: "tag-policy".to_string(),
                    reason: format!("tag '{tag}' is blocked by contract policy"),
                };
            }
        }

        PolicyResult::Allowed
    }

    fn create_contract(&self, commitment: &Commitment) -> BridgeResult<ContractId> {
        let contract_id = ContractId(format!("contract-{}", Uuid::new_v4()));
        self.contracts
            .lock()
            .expect("contracts lock poisoned")
            .insert(commitment.id, contract_id.clone());
        Ok(contract_id)
    }

    fn check_compliance(&self, commitment_id: CommitmentId) -> ComplianceResult {
        let contracts = self.contracts.lock().expect("contracts lock poisoned");
        if contracts.contains_key(&commitment_id) {
            ComplianceResult::Compliant
        } else {
            ComplianceResult::Warning {
                reason: "commitment has no registered contract".to_string(),
            }
        }
    }

    fn request_approval(&self, decision: &Decision) -> BridgeResult<ApprovalStatus> {
        if decision.question.question.trim().is_empty() {
            return Ok(ApprovalStatus::Rejected);
        }

        let risk = self.assess_decision_risk(decision);
        if risk > self.approval_threshold {
            Ok(ApprovalStatus::Pending)
        } else {
            Ok(ApprovalStatus::AutoApproved)
        }
    }
}

#[derive(Debug, Default)]
pub struct MemoryIntegration {
    goal_records: Mutex<HashMap<GoalId, Vec<MemoryRecord>>>,
    decision_records: Mutex<HashMap<DecisionId, Vec<MemoryRecord>>>,
    all_records: Mutex<Vec<MemoryRecord>>,
}

impl MemoryBridge for MemoryIntegration {
    fn persist_goal(&self, goal: &Goal) -> BridgeResult<MemoryId> {
        let record = MemoryRecord {
            id: MemoryId(format!("goal-memory-{}", Uuid::new_v4())),
            summary: format!(
                "Goal: {} ({:?}) progress={:.2}",
                goal.title, goal.status, goal.progress.percentage
            ),
            recorded_at: Timestamp::now(),
        };
        self.goal_records
            .lock()
            .expect("goal records lock poisoned")
            .entry(goal.id)
            .or_default()
            .push(record.clone());
        self.all_records
            .lock()
            .expect("all records lock poisoned")
            .push(record.clone());
        Ok(record.id)
    }

    fn get_goal_context(&self, goal_id: GoalId) -> Vec<MemoryRecord> {
        self.goal_records
            .lock()
            .expect("goal records lock poisoned")
            .get(&goal_id)
            .cloned()
            .unwrap_or_default()
    }

    fn persist_decision(&self, decision: &Decision) -> BridgeResult<MemoryId> {
        let summary = if let Some(chosen) = &decision.chosen {
            format!(
                "Decision: {} -> {}",
                decision.question.question, chosen.name
            )
        } else {
            format!("Decision pending: {}", decision.question.question)
        };
        let record = MemoryRecord {
            id: MemoryId(format!("decision-memory-{}", Uuid::new_v4())),
            summary,
            recorded_at: decision.crystallized_at.unwrap_or_else(Timestamp::now),
        };

        self.decision_records
            .lock()
            .expect("decision records lock poisoned")
            .entry(decision.id)
            .or_default()
            .push(record.clone());
        self.all_records
            .lock()
            .expect("all records lock poisoned")
            .push(record.clone());
        Ok(record.id)
    }

    fn search_context(&self, query: &str) -> Vec<MemoryRecord> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        self.all_records
            .lock()
            .expect("all records lock poisoned")
            .iter()
            .filter(|r| r.summary.to_ascii_lowercase().contains(&q))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct IdentityIntegration {
    known_agents: Mutex<HashSet<String>>,
    accountability: Mutex<HashMap<GoalId, Vec<Receipt>>>,
}

impl IdentityIntegration {
    pub fn with_agents(agent_ids: &[&str]) -> Self {
        Self {
            known_agents: Mutex::new(agent_ids.iter().map(|id| id.to_string()).collect()),
            ..Default::default()
        }
    }
}

impl IdentityBridge for IdentityIntegration {
    fn sign_decision(&self, decision: &Decision) -> BridgeResult<Receipt> {
        let receipt = Receipt {
            id: format!("decision-receipt-{}", decision.id.0),
            signature: format!("sig:decision:{}", decision.id.0),
            signed_at: Timestamp::now(),
        };
        let mut accountability = self
            .accountability
            .lock()
            .expect("accountability lock poisoned");
        for goal_id in &decision.affected_goals {
            accountability
                .entry(*goal_id)
                .or_default()
                .push(receipt.clone());
        }
        Ok(receipt)
    }

    fn sign_commitment(&self, commitment: &Commitment) -> BridgeResult<Receipt> {
        let receipt = Receipt {
            id: format!("commitment-receipt-{}", commitment.id.0),
            signature: format!("sig:commitment:{}", commitment.id.0),
            signed_at: Timestamp::now(),
        };
        if let Some(goal_id) = commitment.goal {
            self.accountability
                .lock()
                .expect("accountability lock poisoned")
                .entry(goal_id)
                .or_default()
                .push(receipt.clone());
        }
        Ok(receipt)
    }

    fn verify_agent(&self, agent_id: &str) -> BridgeResult<AgentIdentity> {
        if agent_id.trim().is_empty() {
            return Err(BridgeError::InvalidInput(
                "agent_id cannot be empty".to_string(),
            ));
        }
        let mut known = self
            .known_agents
            .lock()
            .expect("known agents lock poisoned");
        known.insert(agent_id.to_string());
        Ok(AgentIdentity {
            agent_id: agent_id.to_string(),
            verified_at: Timestamp::now(),
        })
    }

    fn get_accountability_chain(&self, goal_id: GoalId) -> Vec<Receipt> {
        self.accountability
            .lock()
            .expect("accountability lock poisoned")
            .get(&goal_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct CognitionIntegration {
    user_model: UserModel,
}

impl Default for CognitionIntegration {
    fn default() -> Self {
        Self {
            user_model: UserModel {
                focus_areas: vec!["execution".to_string(), "alignment".to_string()],
                risk_tolerance: 0.45,
            },
        }
    }
}

impl CognitionBridge for CognitionIntegration {
    fn get_user_model(&self) -> UserModel {
        self.user_model.clone()
    }

    fn predict_preference(&self, options: &[DecisionPath]) -> Vec<f64> {
        if options.is_empty() {
            return Vec::new();
        }

        let mut raw: Vec<f64> = options
            .iter()
            .map(|opt| {
                let upside = opt.pros.len() as f64;
                let downside = opt.cons.len() as f64;
                (upside + 1.0) / (upside + downside + 2.0)
            })
            .collect();

        let sum: f64 = raw.iter().sum();
        if sum <= f64::EPSILON {
            let equal = 1.0 / options.len() as f64;
            return vec![equal; options.len()];
        }
        for value in &mut raw {
            *value /= sum;
        }
        raw
    }

    fn analyze_patterns(&self, decisions: &[Decision]) -> PatternAnalysis {
        if decisions.is_empty() {
            return PatternAnalysis {
                dominant_pattern: "insufficient-data".to_string(),
                confidence: 0.0,
            };
        }

        let crystallized = decisions
            .iter()
            .filter(|d| d.chosen.is_some() && d.shadows.len() <= 2)
            .count();
        let ratio = crystallized as f64 / decisions.len() as f64;
        PatternAnalysis {
            dominant_pattern: if ratio > 0.65 {
                "decisive-low-branching".to_string()
            } else {
                "exploratory-multi-branch".to_string()
            },
            confidence: ratio.clamp(0.1, 1.0),
        }
    }
}

#[derive(Debug, Default)]
pub struct VisionIntegration {
    linked_evidence: Mutex<HashMap<GoalId, Vec<EvidenceId>>>,
}

impl VisionBridge for VisionIntegration {
    fn capture_evidence(&self, goal_id: GoalId) -> BridgeResult<EvidenceId> {
        Ok(EvidenceId(format!(
            "evidence-{}-{}",
            goal_id.0,
            Uuid::new_v4()
        )))
    }

    fn link_evidence(&self, goal_id: GoalId, evidence_id: EvidenceId) -> BridgeResult<()> {
        self.linked_evidence
            .lock()
            .expect("linked evidence lock poisoned")
            .entry(goal_id)
            .or_default()
            .push(evidence_id);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct NullTimeBridge;

impl TimeBridge for NullTimeBridge {
    fn get_deadline_context(&self, deadline: Timestamp) -> TimeContext {
        let days = (deadline.0 - Timestamp::now().0) as f64 / (86_400.0 * 1e9);
        let urgency_level = if days < 0.0 {
            UrgencyLevel::Overdue
        } else if days < 1.0 {
            UrgencyLevel::Critical
        } else if days < 3.0 {
            UrgencyLevel::High
        } else if days < 7.0 {
            UrgencyLevel::Medium
        } else {
            UrgencyLevel::Low
        };
        TimeContext {
            deadline,
            days_remaining: days,
            is_overdue: days < 0.0,
            urgency_level,
        }
    }

    fn schedule_check(&self, _goal_id: GoalId, _when: Timestamp) -> BridgeResult<()> {
        Ok(())
    }

    fn get_decay(&self, goal: &Goal) -> f64 {
        let neglect = goal.feelings.neglect.clamp(0.0, 1.0);
        let momentum = goal.physics.momentum.clamp(0.0, 1.0);
        (neglect * (1.0 - momentum)).clamp(0.0, 1.0)
    }

    fn calculate_time_urgency(&self, goal: &Goal) -> f64 {
        if let Some(deadline) = goal.deadline {
            match self.get_deadline_context(deadline).urgency_level {
                UrgencyLevel::Overdue => 1.0,
                UrgencyLevel::Critical => 0.95,
                UrgencyLevel::High => 0.8,
                UrgencyLevel::Medium => 0.5,
                UrgencyLevel::Low => 0.2,
            }
        } else {
            0.1
        }
    }
}

#[derive(Debug, Default)]
pub struct NullContractBridge;

impl ContractBridge for NullContractBridge {
    fn check_policy(&self, action: &str, context: &PolicyContext) -> PolicyResult {
        if action.trim().is_empty() {
            return PolicyResult::Denied {
                policy: "action-non-empty".to_string(),
                reason: "action cannot be empty".to_string(),
            };
        }
        if context
            .tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case("forbidden"))
        {
            return PolicyResult::Denied {
                policy: "tag-policy".to_string(),
                reason: "forbidden tag present".to_string(),
            };
        }
        PolicyResult::Allowed
    }

    fn create_contract(&self, commitment: &Commitment) -> BridgeResult<ContractId> {
        Ok(ContractId(format!("contract-{}", commitment.id.0)))
    }

    fn check_compliance(&self, _commitment_id: CommitmentId) -> ComplianceResult {
        ComplianceResult::Compliant
    }

    fn request_approval(&self, decision: &Decision) -> BridgeResult<ApprovalStatus> {
        if decision.question.question.trim().is_empty() {
            return Ok(ApprovalStatus::Rejected);
        }
        Ok(ApprovalStatus::AutoApproved)
    }
}

#[derive(Debug, Default)]
pub struct NullMemoryBridge;

impl MemoryBridge for NullMemoryBridge {
    fn persist_goal(&self, goal: &Goal) -> BridgeResult<MemoryId> {
        Ok(MemoryId(format!("goal-{}", goal.id.0)))
    }

    fn get_goal_context(&self, goal_id: GoalId) -> Vec<MemoryRecord> {
        vec![MemoryRecord {
            id: MemoryId(format!("goal-{goal_id:?}")),
            summary: "No historical context in null bridge".to_string(),
            recorded_at: Timestamp::now(),
        }]
    }

    fn persist_decision(&self, decision: &Decision) -> BridgeResult<MemoryId> {
        Ok(MemoryId(format!("decision-{}", decision.id.0)))
    }

    fn search_context(&self, query: &str) -> Vec<MemoryRecord> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        vec![MemoryRecord {
            id: MemoryId(format!("search-{}", Uuid::new_v4())),
            summary: format!("No concrete matches for '{query}' in null bridge"),
            recorded_at: Timestamp::now(),
        }]
    }
}

#[derive(Debug, Default)]
pub struct NullIdentityBridge;

impl IdentityBridge for NullIdentityBridge {
    fn sign_decision(&self, decision: &Decision) -> BridgeResult<Receipt> {
        Ok(Receipt {
            id: format!("decision-receipt-{}", decision.id.0),
            signature: "null-signature".to_string(),
            signed_at: Timestamp::now(),
        })
    }

    fn sign_commitment(&self, commitment: &Commitment) -> BridgeResult<Receipt> {
        Ok(Receipt {
            id: format!("commitment-receipt-{}", commitment.id.0),
            signature: "null-signature".to_string(),
            signed_at: Timestamp::now(),
        })
    }

    fn verify_agent(&self, agent_id: &str) -> BridgeResult<AgentIdentity> {
        if agent_id.trim().is_empty() {
            return Err(BridgeError::InvalidInput(
                "agent_id cannot be empty".to_string(),
            ));
        }
        Ok(AgentIdentity {
            agent_id: agent_id.to_string(),
            verified_at: Timestamp::now(),
        })
    }

    fn get_accountability_chain(&self, goal_id: GoalId) -> Vec<Receipt> {
        vec![Receipt {
            id: format!("accountability-{goal_id:?}"),
            signature: "null-signature".to_string(),
            signed_at: Timestamp::now(),
        }]
    }
}

#[derive(Debug, Default)]
pub struct NullCognitionBridge;

impl CognitionBridge for NullCognitionBridge {
    fn get_user_model(&self) -> UserModel {
        UserModel {
            focus_areas: vec!["execution".to_string(), "reliability".to_string()],
            risk_tolerance: 0.5,
        }
    }

    fn predict_preference(&self, options: &[DecisionPath]) -> Vec<f64> {
        if options.is_empty() {
            return Vec::new();
        }
        let weight = 1.0 / options.len() as f64;
        vec![weight; options.len()]
    }

    fn analyze_patterns(&self, decisions: &[Decision]) -> PatternAnalysis {
        PatternAnalysis {
            dominant_pattern: if decisions.is_empty() {
                "insufficient-data".to_string()
            } else {
                "single-pass-crystallization".to_string()
            },
            confidence: if decisions.is_empty() { 0.0 } else { 0.6 },
        }
    }
}

#[derive(Debug, Default)]
pub struct NullVisionBridge;

impl VisionBridge for NullVisionBridge {
    fn capture_evidence(&self, goal_id: GoalId) -> BridgeResult<EvidenceId> {
        Ok(EvidenceId(format!(
            "evidence-{goal_id:?}-{}",
            Uuid::new_v4()
        )))
    }

    fn link_evidence(&self, _goal_id: GoalId, _evidence_id: EvidenceId) -> BridgeResult<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct NullCodebaseBridge;

impl CodebaseBridge for NullCodebaseBridge {}

#[derive(Debug, Default)]
pub struct NullCommBridge;

impl CommBridge for NullCommBridge {}

// ---------------------------------------------------------------------------
// Concrete codebase / comm integrations (placeholder state-tracking impls)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CodebaseIntegration {
    linked_symbols: Mutex<HashMap<GoalId, Vec<String>>>,
}

impl CodebaseBridge for CodebaseIntegration {
    fn link_goal_to_symbol(&self, goal_id: GoalId, symbol_name: &str) -> BridgeResult<()> {
        self.linked_symbols
            .lock()
            .expect("linked symbols lock poisoned")
            .entry(goal_id)
            .or_default()
            .push(symbol_name.to_string());
        Ok(())
    }

    fn find_related_code(&self, topic: &str, max_results: usize) -> Vec<String> {
        let _ = max_results;
        let linked = self
            .linked_symbols
            .lock()
            .expect("linked symbols lock poisoned");
        let q = topic.to_ascii_lowercase();
        linked
            .values()
            .flat_map(|syms| syms.iter())
            .filter(|s| s.to_ascii_lowercase().contains(&q))
            .take(max_results)
            .cloned()
            .collect()
    }

    fn code_context(&self, symbol_name: &str) -> Option<String> {
        let linked = self
            .linked_symbols
            .lock()
            .expect("linked symbols lock poisoned");
        let found = linked
            .values()
            .any(|syms| syms.iter().any(|s| s == symbol_name));
        if found {
            Some(format!(
                "Symbol '{symbol_name}' is linked to a planning goal"
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct CommIntegration {
    broadcasts: Mutex<Vec<(String, GoalId)>>,
    notifications: Mutex<Vec<(CommitmentId, String)>>,
}

impl CommBridge for CommIntegration {
    fn broadcast_event(&self, event_type: &str, goal_id: GoalId) -> BridgeResult<()> {
        self.broadcasts
            .lock()
            .expect("broadcasts lock poisoned")
            .push((event_type.to_string(), goal_id));
        Ok(())
    }

    fn notify_stakeholders(&self, commitment_id: CommitmentId, message: &str) -> BridgeResult<()> {
        self.notifications
            .lock()
            .expect("notifications lock poisoned")
            .push((commitment_id, message.to_string()));
        Ok(())
    }

    fn store_from_channel(&self, _channel_id: u64, summary: &str) -> BridgeResult<MemoryId> {
        Ok(MemoryId(format!(
            "comm-{}-{}",
            summary.len(),
            Uuid::new_v4()
        )))
    }
}

// ---------------------------------------------------------------------------
// Unified NoOpBridges for standalone use (parity with all other sisters)
// ---------------------------------------------------------------------------

/// No-op implementation of all bridges for standalone use.
/// Matches the standard sister pattern where a single struct implements every bridge.
#[derive(Debug, Clone, Default)]
pub struct NoOpBridges;

impl TimeBridge for NoOpBridges {
    fn get_deadline_context(&self, deadline: Timestamp) -> TimeContext {
        NullTimeBridge.get_deadline_context(deadline)
    }
    fn schedule_check(&self, goal_id: GoalId, when: Timestamp) -> BridgeResult<()> {
        NullTimeBridge.schedule_check(goal_id, when)
    }
    fn get_decay(&self, goal: &Goal) -> f64 {
        NullTimeBridge.get_decay(goal)
    }
    fn calculate_time_urgency(&self, goal: &Goal) -> f64 {
        NullTimeBridge.calculate_time_urgency(goal)
    }
}

impl ContractBridge for NoOpBridges {
    fn check_policy(&self, action: &str, context: &PolicyContext) -> PolicyResult {
        NullContractBridge.check_policy(action, context)
    }
    fn create_contract(&self, commitment: &Commitment) -> BridgeResult<ContractId> {
        NullContractBridge.create_contract(commitment)
    }
    fn check_compliance(&self, commitment_id: CommitmentId) -> ComplianceResult {
        NullContractBridge.check_compliance(commitment_id)
    }
    fn request_approval(&self, decision: &Decision) -> BridgeResult<ApprovalStatus> {
        NullContractBridge.request_approval(decision)
    }
}

impl MemoryBridge for NoOpBridges {
    fn persist_goal(&self, goal: &Goal) -> BridgeResult<MemoryId> {
        NullMemoryBridge.persist_goal(goal)
    }
    fn get_goal_context(&self, goal_id: GoalId) -> Vec<MemoryRecord> {
        NullMemoryBridge.get_goal_context(goal_id)
    }
    fn persist_decision(&self, decision: &Decision) -> BridgeResult<MemoryId> {
        NullMemoryBridge.persist_decision(decision)
    }
    fn search_context(&self, query: &str) -> Vec<MemoryRecord> {
        NullMemoryBridge.search_context(query)
    }
}

impl IdentityBridge for NoOpBridges {
    fn sign_decision(&self, decision: &Decision) -> BridgeResult<Receipt> {
        NullIdentityBridge.sign_decision(decision)
    }
    fn sign_commitment(&self, commitment: &Commitment) -> BridgeResult<Receipt> {
        NullIdentityBridge.sign_commitment(commitment)
    }
    fn verify_agent(&self, agent_id: &str) -> BridgeResult<AgentIdentity> {
        NullIdentityBridge.verify_agent(agent_id)
    }
    fn get_accountability_chain(&self, goal_id: GoalId) -> Vec<Receipt> {
        NullIdentityBridge.get_accountability_chain(goal_id)
    }
}

impl CognitionBridge for NoOpBridges {
    fn get_user_model(&self) -> UserModel {
        NullCognitionBridge.get_user_model()
    }
    fn predict_preference(&self, options: &[DecisionPath]) -> Vec<f64> {
        NullCognitionBridge.predict_preference(options)
    }
    fn analyze_patterns(&self, decisions: &[Decision]) -> PatternAnalysis {
        NullCognitionBridge.analyze_patterns(decisions)
    }
}

impl VisionBridge for NoOpBridges {
    fn capture_evidence(&self, goal_id: GoalId) -> BridgeResult<EvidenceId> {
        NullVisionBridge.capture_evidence(goal_id)
    }
    fn link_evidence(&self, goal_id: GoalId, evidence_id: EvidenceId) -> BridgeResult<()> {
        NullVisionBridge.link_evidence(goal_id, evidence_id)
    }
}

impl CodebaseBridge for NoOpBridges {}
impl CommBridge for NoOpBridges {}

// ---------------------------------------------------------------------------
// Bridge configuration (parity with all other sisters)
// ---------------------------------------------------------------------------

/// Configuration for which bridges are active.
#[derive(Debug, Clone, Default)]
pub struct BridgeConfig {
    pub time_enabled: bool,
    pub contract_enabled: bool,
    pub memory_enabled: bool,
    pub identity_enabled: bool,
    pub cognition_enabled: bool,
    pub vision_enabled: bool,
    pub codebase_enabled: bool,
    pub comm_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentic_planning::{
        CreateCommitmentRequest, CreateDecisionRequest, CreateGoalRequest, PlanningEngine, Promise,
        Stakeholder, StakeholderId,
    };

    #[test]
    fn bridge_smoke() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: "Bridge goal".to_string(),
                intention: "bridge".to_string(),
                deadline: Some(Timestamp::days_from_now(2.0)),
                ..Default::default()
            })
            .expect("create goal");
        let decision = engine
            .create_decision(CreateDecisionRequest {
                question: "Use bridge?".to_string(),
                ..Default::default()
            })
            .expect("create decision");
        let commitment = engine
            .create_commitment(CreateCommitmentRequest {
                promise: Promise {
                    description: "Ship bridge integration".to_string(),
                    ..Default::default()
                },
                stakeholder: Stakeholder {
                    id: StakeholderId(Uuid::new_v4()),
                    name: "Integrator".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .expect("create commitment");

        let time = NullTimeBridge;
        let ctx = time.get_deadline_context(Timestamp::days_from_now(3.0));
        assert!(ctx.days_remaining > 0.0);
        assert!(time.calculate_time_urgency(&goal) > 0.1);
        assert!(time.get_decay(&goal) >= 0.0);

        let contract = NullContractBridge;
        let policy = contract.check_policy(
            "goal.progress",
            &PolicyContext {
                goal: Some(goal.id),
                decision: Some(decision.id),
                tags: vec!["planning".to_string()],
            },
        );
        assert!(matches!(policy, PolicyResult::Allowed));
        assert!(contract.create_contract(&commitment).is_ok());
        assert!(matches!(
            contract.check_compliance(commitment.id),
            ComplianceResult::Compliant
        ));
        assert!(matches!(
            contract.request_approval(&decision).expect("approval"),
            ApprovalStatus::AutoApproved
        ));

        let memory = NullMemoryBridge;
        assert!(memory.persist_goal(&goal).is_ok());
        assert!(memory.persist_decision(&decision).is_ok());
        assert!(!memory.get_goal_context(goal.id).is_empty());
        assert!(!memory.search_context("bridge").is_empty());

        let identity = NullIdentityBridge;
        assert!(identity.sign_decision(&decision).is_ok());
        assert!(identity.sign_commitment(&commitment).is_ok());
        assert!(identity.verify_agent("agent-alpha").is_ok());
        assert!(!identity.get_accountability_chain(goal.id).is_empty());

        let cognition = NullCognitionBridge;
        assert!(cognition.get_user_model().risk_tolerance >= 0.0);
        assert_eq!(cognition.predict_preference(&[]).len(), 0);
        assert!(cognition.analyze_patterns(&[]).confidence <= 1.0);

        let vision = NullVisionBridge;
        let evidence = vision.capture_evidence(goal.id).expect("capture evidence");
        assert!(vision.link_evidence(goal.id, evidence).is_ok());

        let codebase = NullCodebaseBridge;
        assert!(codebase.link_goal_to_symbol(goal.id, "my_func").is_err());
        assert!(codebase.find_related_code("topic", 5).is_empty());
        assert!(codebase.code_context("my_func").is_none());

        let comm = NullCommBridge;
        assert!(comm.broadcast_event("goal.created", goal.id).is_err());
        assert!(comm.notify_stakeholders(commitment.id, "test").is_err());
        assert!(comm.store_from_channel(1, "summary").is_err());
    }

    #[test]
    fn hydra_adapter_handles_core_requests() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: "Hydra goal".to_string(),
                intention: "validate adapter".to_string(),
                ..Default::default()
            })
            .expect("create goal");
        engine.activate_goal(goal.id).expect("activate goal");

        let mut adapter = PlanningHydraAdapter::new(engine);
        assert_eq!(adapter.id(), "agentic-planning");
        assert_eq!(adapter.adapter_type(), AdapterType::Sister);
        assert_eq!(adapter.capabilities().len(), 5);

        let health = adapter.health();
        assert!(matches!(
            health,
            HealthStatus::Healthy {
                goals: 1,
                decisions: 0,
                commitments: 0
            }
        ));

        let active = adapter
            .handle(AdapterRequest::GetActiveGoals)
            .expect("active");
        match active {
            AdapterResponse::Goals(goals) => {
                assert_eq!(goals.len(), 1);
                assert_eq!(goals[0].id, goal.id);
            }
            _ => panic!("expected goal list response"),
        }

        let singularity = adapter
            .handle(AdapterRequest::GetSingularity)
            .expect("singularity");
        match singularity {
            AdapterResponse::Singularity(s) => {
                assert_eq!(s.goal_positions.len(), 1);
                assert!(s.goal_positions.contains_key(&goal.id));
            }
            _ => panic!("expected singularity response"),
        }

        let progressed = adapter
            .handle(AdapterRequest::RecordProgress {
                goal_id: goal.id,
                percentage: 0.45,
                note: Some("hydra progress".to_string()),
            })
            .expect("progress");
        match progressed {
            AdapterResponse::Goal(updated) => {
                assert_eq!(updated.id, goal.id);
                assert!((updated.progress.percentage - 0.45).abs() < f64::EPSILON);
            }
            _ => panic!("expected goal response"),
        }

        let blockers = adapter
            .handle(AdapterRequest::GetBlockers)
            .expect("blockers");
        match blockers {
            AdapterResponse::Blockers(values) => assert!(values.is_empty()),
            _ => panic!("expected blocker response"),
        }
    }

    #[test]
    fn concrete_integrations_track_state() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: "Integration goal".to_string(),
                intention: "exercise concrete bridges".to_string(),
                deadline: Some(Timestamp::days_from_now(2.0)),
                ..Default::default()
            })
            .expect("create goal");
        let decision = engine
            .create_decision(CreateDecisionRequest {
                question: "Production rollout decision".to_string(),
                goals: Some(vec![goal.id]),
                constraints: Some(vec![
                    "must meet regulatory controls".to_string(),
                    "requires legal sign-off".to_string(),
                ]),
                ..Default::default()
            })
            .expect("create decision");
        let decision = engine
            .add_option(
                decision.id,
                DecisionPath {
                    name: "Option A".to_string(),
                    pros: vec!["faster".to_string()],
                    cons: vec!["risk".to_string()],
                    ..Default::default()
                },
            )
            .expect("add option A")
            .clone();
        let decision = engine
            .add_option(
                decision.id,
                DecisionPath {
                    name: "Option B".to_string(),
                    pros: vec!["safer".to_string()],
                    cons: vec!["slower".to_string()],
                    ..Default::default()
                },
            )
            .expect("add option B")
            .clone();
        let commitment = engine
            .create_commitment(CreateCommitmentRequest {
                promise: Promise {
                    description: "Ship integration-safe release".to_string(),
                    ..Default::default()
                },
                stakeholder: Stakeholder {
                    id: StakeholderId(Uuid::new_v4()),
                    name: "Ops".to_string(),
                    ..Default::default()
                },
                goal: Some(goal.id),
                ..Default::default()
            })
            .expect("create commitment");

        let time = TimeIntegration::default();
        time.schedule_check(goal.id, Timestamp::days_from_now(1.0))
            .expect("schedule check");
        assert_eq!(time.scheduled_checks().len(), 1);
        assert!(time.calculate_time_urgency(&goal) >= 0.2);

        let contract = ContractIntegration::default();
        assert!(matches!(
            contract.check_policy(
                "goal.progress",
                &PolicyContext {
                    goal: Some(goal.id),
                    decision: Some(decision.id),
                    tags: vec!["planning".to_string()],
                }
            ),
            PolicyResult::Allowed
        ));
        let _contract_id = contract
            .create_contract(&commitment)
            .expect("create contract");
        assert!(matches!(
            contract.check_compliance(commitment.id),
            ComplianceResult::Compliant
        ));
        assert!(matches!(
            contract.request_approval(&decision).expect("approval"),
            ApprovalStatus::Pending
        ));

        let memory = MemoryIntegration::default();
        memory.persist_goal(&goal).expect("persist goal");
        memory
            .persist_decision(&decision)
            .expect("persist decision");
        assert!(!memory.get_goal_context(goal.id).is_empty());
        assert!(!memory.search_context("integration").is_empty());

        let identity = IdentityIntegration::default();
        identity.verify_agent("agent-alpha").expect("verify agent");
        identity
            .sign_decision(&decision)
            .expect("sign decision receipt");
        identity
            .sign_commitment(&commitment)
            .expect("sign commitment receipt");
        assert!(identity.get_accountability_chain(goal.id).len() >= 2);

        let vision = VisionIntegration::default();
        let evidence = vision.capture_evidence(goal.id).expect("capture evidence");
        vision
            .link_evidence(goal.id, evidence)
            .expect("link evidence");

        let cognition = CognitionIntegration::default();
        let prefs = cognition.predict_preference(
            &decision
                .shadows
                .iter()
                .map(|s| s.path.clone())
                .collect::<Vec<_>>(),
        );
        assert!(prefs.len() <= decision.shadows.len());
        assert!((0.0..=1.0).contains(&cognition.get_user_model().risk_tolerance));

        // --- New: codebase and comm concrete integrations ---
        let codebase = CodebaseIntegration::default();
        codebase
            .link_goal_to_symbol(goal.id, "PlanningEngine::create_goal")
            .expect("link goal to symbol");
        let related = codebase.find_related_code("planning", 10);
        assert_eq!(related.len(), 1);
        assert!(codebase
            .code_context("PlanningEngine::create_goal")
            .is_some());
        assert!(codebase.code_context("nonexistent").is_none());

        let comm = CommIntegration::default();
        comm.broadcast_event("goal.created", goal.id)
            .expect("broadcast event");
        comm.notify_stakeholders(commitment.id, "commitment updated")
            .expect("notify stakeholders");
        assert!(comm.store_from_channel(42, "planning discussion").is_ok());
    }

    #[test]
    fn noop_bridges_implements_all_traits() {
        let b = NoOpBridges;
        // Verify NoOpBridges implements every bridge trait
        let _: &dyn TimeBridge = &b;
        let _: &dyn ContractBridge = &b;
        let _: &dyn MemoryBridge = &b;
        let _: &dyn IdentityBridge = &b;
        let _: &dyn CognitionBridge = &b;
        let _: &dyn VisionBridge = &b;
        let _: &dyn CodebaseBridge = &b;
        let _: &dyn CommBridge = &b;
    }

    #[test]
    fn noop_bridges_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoOpBridges>();
        assert_send_sync::<NullTimeBridge>();
        assert_send_sync::<NullContractBridge>();
        assert_send_sync::<NullMemoryBridge>();
        assert_send_sync::<NullIdentityBridge>();
        assert_send_sync::<NullCognitionBridge>();
        assert_send_sync::<NullVisionBridge>();
        assert_send_sync::<NullCodebaseBridge>();
        assert_send_sync::<NullCommBridge>();
    }

    #[test]
    fn noop_bridges_default_and_clone() {
        let b = NoOpBridges;
        let _b2 = b.clone();
    }

    #[test]
    fn bridge_config_defaults_all_false() {
        let cfg = BridgeConfig::default();
        assert!(!cfg.time_enabled);
        assert!(!cfg.contract_enabled);
        assert!(!cfg.memory_enabled);
        assert!(!cfg.identity_enabled);
        assert!(!cfg.cognition_enabled);
        assert!(!cfg.vision_enabled);
        assert!(!cfg.codebase_enabled);
        assert!(!cfg.comm_enabled);
    }

    #[test]
    fn noop_bridges_delegates_correctly() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: "NoOp test goal".to_string(),
                intention: "verify noop delegation".to_string(),
                deadline: Some(Timestamp::days_from_now(5.0)),
                ..Default::default()
            })
            .expect("create goal");
        let decision = engine
            .create_decision(CreateDecisionRequest {
                question: "NoOp test?".to_string(),
                ..Default::default()
            })
            .expect("create decision");
        let commitment = engine
            .create_commitment(CreateCommitmentRequest {
                promise: Promise {
                    description: "NoOp commitment".to_string(),
                    ..Default::default()
                },
                stakeholder: Stakeholder {
                    id: StakeholderId(Uuid::new_v4()),
                    name: "Tester".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .expect("create commitment");

        let b = NoOpBridges;

        // Time
        let ctx = b.get_deadline_context(Timestamp::days_from_now(3.0));
        assert!(ctx.days_remaining > 0.0);
        assert!(b
            .schedule_check(goal.id, Timestamp::days_from_now(1.0))
            .is_ok());
        assert!(b.get_decay(&goal) >= 0.0);
        assert!(b.calculate_time_urgency(&goal) >= 0.1);

        // Contract
        assert!(matches!(
            b.check_policy(
                "test",
                &PolicyContext {
                    goal: Some(goal.id),
                    decision: None,
                    tags: vec![],
                }
            ),
            PolicyResult::Allowed
        ));
        assert!(b.create_contract(&commitment).is_ok());
        assert!(matches!(
            b.check_compliance(commitment.id),
            ComplianceResult::Compliant
        ));
        assert!(b.request_approval(&decision).is_ok());

        // Memory
        assert!(b.persist_goal(&goal).is_ok());
        assert!(b.persist_decision(&decision).is_ok());
        assert!(!b.get_goal_context(goal.id).is_empty());
        assert!(!b.search_context("noop").is_empty());

        // Identity
        assert!(b.sign_decision(&decision).is_ok());
        assert!(b.sign_commitment(&commitment).is_ok());
        assert!(b.verify_agent("agent-1").is_ok());
        assert!(!b.get_accountability_chain(goal.id).is_empty());

        // Cognition
        assert!(b.get_user_model().risk_tolerance >= 0.0);
        assert_eq!(b.predict_preference(&[]).len(), 0);
        assert!(b.analyze_patterns(&[]).confidence <= 1.0);

        // Vision
        let eid = b.capture_evidence(goal.id).expect("capture");
        assert!(b.link_evidence(goal.id, eid).is_ok());

        // Codebase (default no-op returns errors)
        assert!(b.link_goal_to_symbol(goal.id, "sym").is_err());
        assert!(b.find_related_code("topic", 5).is_empty());
        assert!(b.code_context("sym").is_none());

        // Comm (default no-op returns errors)
        assert!(b.broadcast_event("test", goal.id).is_err());
        assert!(b.notify_stakeholders(commitment.id, "msg").is_err());
        assert!(b.store_from_channel(1, "test").is_err());
    }
}
