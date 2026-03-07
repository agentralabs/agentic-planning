pub mod audit;
pub mod auth;
pub mod bridges;
pub mod cache;
pub mod contracts;
mod error;
mod file_format;
#[path = "indexes.rs"]
mod indexes;
mod inventions;
pub mod isolation;
pub mod locking;
pub mod metrics;
pub mod query;
mod query_engine;
pub mod types;
mod validation;
mod write_engine;

pub use audit::{AuditAction, AuditEntityType, AuditEntry, AuditLog};
pub use error::{Error, Result};
pub use indexes::PlanIndexes;
pub use types::*;
pub use validation::{validators, ValidationError, ValidationResult};

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PlanningEngine {
    pub(crate) path: Option<PathBuf>,
    pub(crate) dirty: bool,
    pub(crate) goal_store: HashMap<GoalId, Goal>,
    pub(crate) decision_store: HashMap<DecisionId, Decision>,
    pub(crate) commitment_store: HashMap<CommitmentId, Commitment>,
    pub(crate) dream_store: HashMap<DreamId, Dream>,
    pub(crate) federation_store: HashMap<FederationId, Federation>,
    pub(crate) soul_archive: HashMap<GoalId, GoalSoulArchive>,
    pub(crate) consensus_store: HashMap<DecisionId, DecisionConsensus>,
    pub(crate) indexes: PlanIndexes,
    pub(crate) audit_log: AuditLog,
    pub(crate) write_count: u64,
    pub(crate) session_id: uuid::Uuid,
}

impl PlanningEngine {
    pub fn in_memory() -> Self {
        Self {
            path: None,
            dirty: false,
            goal_store: HashMap::new(),
            decision_store: HashMap::new(),
            commitment_store: HashMap::new(),
            dream_store: HashMap::new(),
            federation_store: HashMap::new(),
            soul_archive: HashMap::new(),
            consensus_store: HashMap::new(),
            indexes: PlanIndexes::new(),
            audit_log: AuditLog::new(),
            write_count: 0,
            session_id: uuid::Uuid::new_v4(),
        }
    }

    pub fn goal_count(&self) -> usize {
        self.goal_store.len()
    }

    pub fn decision_count(&self) -> usize {
        self.decision_store.len()
    }

    pub fn commitment_count(&self) -> usize {
        self.commitment_store.len()
    }

    pub fn session_id(&self) -> uuid::Uuid {
        self.session_id
    }

    pub fn audit_log_mut(&mut self) -> &mut AuditLog {
        &mut self.audit_log
    }

    /// Iterate over all goals (for ghost bridge context).
    pub fn goals(&self) -> impl Iterator<Item = &Goal> {
        self.goal_store.values()
    }

    /// Iterate over all decisions (for ghost bridge context).
    pub fn decisions(&self) -> impl Iterator<Item = &Decision> {
        self.decision_store.values()
    }

    /// Iterate over all commitments (for ghost bridge context).
    pub fn commitments(&self) -> impl Iterator<Item = &Commitment> {
        self.commitment_store.values()
    }

    pub(crate) fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    #[allow(dead_code)]
    pub(crate) fn rebuild_indexes(&mut self) {
        self.indexes.rebuild_full(
            &self.goal_store,
            &self.decision_store,
            &self.commitment_store,
            &self.dream_store,
            &self.federation_store,
        );
    }

    pub(crate) fn calculate_initial_urgency(&self, request: &CreateGoalRequest) -> f64 {
        let priority: f64 = match request.priority.unwrap_or(Priority::Medium) {
            Priority::Critical => 1.0,
            Priority::High => 0.8,
            Priority::Medium => 0.5,
            Priority::Low => 0.3,
            Priority::Someday => 0.1,
        };

        let deadline_factor: f64 = request
            .deadline
            .map(|d| {
                let days = (d.0 - Timestamp::now().0) as f64 / (86_400.0 * 1e9);
                if days <= 1.0 {
                    1.0
                } else if days <= 7.0 {
                    0.8
                } else if days <= 30.0 {
                    0.6
                } else {
                    0.3
                }
            })
            .unwrap_or(0.4);

        ((priority + deadline_factor) / 2.0).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_initial_gravity(&self, request: &CreateGoalRequest) -> f64 {
        let emotional_weight = request.emotional_weight.unwrap_or(0.5);
        let priority = match request.priority.unwrap_or(Priority::Medium) {
            Priority::Critical => 0.95,
            Priority::High => 0.8,
            Priority::Medium => 0.6,
            Priority::Low => 0.4,
            Priority::Someday => 0.2,
        };
        ((emotional_weight + priority) / 2.0).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_initial_inertia(&self, request: &CreateGoalRequest) -> f64 {
        let mut inertia = 0.35;
        if let Some(deps) = &request.dependencies {
            inertia += (deps.len() as f64 * 0.08).min(0.4);
        }
        if request.parent.is_some() {
            inertia += 0.1;
        }
        inertia.clamp(0.1, 1.0)
    }

    pub(crate) fn calculate_velocity(&self, history: &[ProgressPoint]) -> f64 {
        if history.len() < 2 {
            return 0.0;
        }

        let first = &history[0];
        let last = &history[history.len() - 1];
        let dt_days = ((last.timestamp.0 - first.timestamp.0) as f64 / (86_400.0 * 1e9)).max(1.0);
        let dp = (last.percentage - first.percentage).max(0.0);
        (dp / dt_days).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_momentum_from_goal(&self, goal: &Goal) -> f64 {
        let recent_factor = goal
            .progress
            .history
            .last()
            .map(|p| {
                let age_days =
                    ((Timestamp::now().0 - p.timestamp.0) as f64 / (86_400.0 * 1e9)).max(0.0);
                (1.0 - (age_days / 30.0)).clamp(0.0, 1.0)
            })
            .unwrap_or(0.0);
        (goal.progress.velocity * 0.6 + recent_factor * 0.4).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_confidence_from_goal(&self, goal: &Goal) -> f64 {
        let blocker_penalty: f64 = goal
            .blockers
            .iter()
            .filter(|b| b.resolved_at.is_none())
            .map(|b| b.severity)
            .sum::<f64>()
            .min(1.0);
        let progress_boost = goal.progress.percentage * 0.5;
        (0.5 + progress_boost - blocker_penalty * 0.6).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_reincarnation_potential(&self, goal: &Goal) -> f64 {
        let progress_component = goal.progress.percentage;
        let soul_component = goal.soul.emotional_weight;
        ((progress_component + soul_component) / 2.0).clamp(0.0, 1.0)
    }

    #[allow(dead_code)]
    pub(crate) fn check_success_criteria(&mut self, goal: &mut Goal) {
        let now = Timestamp::now();
        for c in &mut goal.soul.success_criteria {
            if c.achieved {
                continue; // already achieved, skip
            }

            let met = if c.measurable {
                // For measurable criteria with a target, check against goal progress
                match c.target {
                    Some(target) => goal.progress.percentage >= target,
                    None => goal.progress.percentage >= 1.0,
                }
            } else {
                // Non-measurable criteria: consider achieved only if goal is fully complete
                goal.progress.percentage >= 1.0
            };

            if met {
                c.achieved = true;
                c.achieved_at = Some(now);
            }
        }
    }

    pub(crate) fn release_completion_energy(&mut self, goal_id: GoalId) {
        let dependent_ids = self
            .goal_store
            .get(&goal_id)
            .map(|g| g.dependents.clone())
            .unwrap_or_default();

        for dep_id in dependent_ids {
            if let Some(dep) = self.goal_store.get_mut(&dep_id) {
                dep.physics.energy = (dep.physics.energy + 0.2).clamp(0.0, 2.0);
                dep.physics.momentum = (dep.physics.momentum + 0.1).clamp(0.0, 1.0);
            }
        }
    }

    pub(crate) fn check_unblock(&mut self, goal_id: &GoalId) {
        let should_unblock = self
            .goal_store
            .get(goal_id)
            .map(|g| {
                g.status == GoalStatus::Blocked
                    && g.blockers.iter().all(|b| b.resolved_at.is_some())
                    && g.dependencies.iter().all(|d| {
                        self.goal_store
                            .get(d)
                            .map(|dg| dg.status == GoalStatus::Completed)
                            .unwrap_or(false)
                    })
            })
            .unwrap_or(false);

        if should_unblock {
            if let Some(goal) = self.goal_store.get_mut(goal_id) {
                goal.status = GoalStatus::Active;
            }
        }
    }

    pub(crate) fn calculate_resurrection_cost(
        &self,
        decision: &Decision,
        shadow: &CrystalShadow,
    ) -> f64 {
        let base = if decision.status == DecisionStatus::Crystallized {
            0.6
        } else {
            0.3
        };
        let complexity = (shadow.path.pros.len() + shadow.path.cons.len()) as f64 * 0.03;
        (base + complexity).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_reversibility(&self, decision: &Decision) -> Reversibility {
        let complexity = decision.shadows.len() as f64;
        Reversibility {
            is_reversible: complexity < 6.0,
            reversal_cost: (0.2 + complexity * 0.1).clamp(0.0, 1.0),
            reversal_window: Some(Timestamp::days_from_now(14.0)),
            cascade_count: decision.causes.len(),
        }
    }

    pub(crate) fn calculate_regret(&self, decision: &Decision) -> f64 {
        let negative = decision
            .consequences
            .iter()
            .filter(|c| matches!(c.impact, Impact::Negative))
            .count() as f64;
        let total = decision.consequences.len().max(1) as f64;
        (negative / total).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_commitment_weight(&self, request: &CreateCommitmentRequest) -> f64 {
        let stakeholder = request.stakeholder.importance.clamp(0.0, 1.0);
        let complexity = (request.promise.deliverables.len() as f64 * 0.05).clamp(0.0, 0.4);
        (0.3 + stakeholder * 0.5 + complexity).clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_commitment_inertia(&self, request: &CreateCommitmentRequest) -> f64 {
        let due_pressure: f64 = request
            .due
            .map(|d| {
                let days = (d.0 - Timestamp::now().0) as f64 / (86_400.0 * 1e9);
                if days <= 7.0 {
                    0.7
                } else {
                    0.4
                }
            })
            .unwrap_or(0.3);
        due_pressure.clamp(0.0, 1.0)
    }

    pub(crate) fn calculate_breaking_cost(
        &self,
        request: &CreateCommitmentRequest,
    ) -> BreakingCost {
        let trust = request.stakeholder.importance.clamp(0.0, 1.0);
        BreakingCost {
            trust_damage: trust,
            relationship_impact: (trust * 0.8).clamp(0.0, 1.0),
            reputation_cost: (trust * 0.6).clamp(0.0, 1.0),
            energy_to_break: 0.5,
            cascading_effects: Vec::new(),
        }
    }

    pub(crate) fn calculate_chain_bonus(&self, id: CommitmentId) -> f64 {
        let commitment = match self.commitment_store.get(&id) {
            Some(c) => c,
            None => return 0.0,
        };

        // Walk entanglement chain: fulfilled neighbors boost, broken neighbors penalize
        let mut fulfilled_count: usize = 0;
        let mut broken_count: usize = 0;

        for entanglement in &commitment.entanglements {
            if let Some(linked) = self.commitment_store.get(&entanglement.with) {
                match linked.status {
                    CommitmentStatus::Fulfilled => fulfilled_count += 1,
                    CommitmentStatus::Broken => broken_count += 1,
                    _ => {}
                }
            }
        }

        let bonus = fulfilled_count as f64 * 0.05 - broken_count as f64 * 0.02;
        bonus.clamp(-0.1, 0.3)
    }

    pub(crate) fn boost_commitment(&mut self, id: CommitmentId, bonus: f64) -> Result<()> {
        let c = self
            .commitment_store
            .get_mut(&id)
            .ok_or(Error::CommitmentNotFound(id))?;
        c.weight = (c.weight + bonus).clamp(0.0, 1.0);
        Ok(())
    }

    pub(crate) fn destabilize_commitment(&mut self, id: CommitmentId) -> Result<()> {
        let c = self
            .commitment_store
            .get_mut(&id)
            .ok_or(Error::CommitmentNotFound(id))?;
        c.status = CommitmentStatus::AtRisk;
        Ok(())
    }

    pub(crate) fn generate_completion_scenario(&self, goal: &Goal) -> CompletionScenario {
        CompletionScenario {
            vision: format!(
                "{} has been achieved. Success criteria are met.",
                goal.title
            ),
            feeling: "Clarity and momentum".to_string(),
            world_changes: goal
                .soul
                .success_criteria
                .iter()
                .map(|c| format!("✓ {}", c.description))
                .collect(),
            stakeholder_reactions: HashMap::new(),
        }
    }

    pub(crate) fn predict_obstacles(&self, goal: &Goal) -> Vec<DreamObstacle> {
        let mut obstacles = Vec::new();

        for dep_id in &goal.dependencies {
            if let Some(dep) = self.goal_store.get(dep_id) {
                if dep.status != GoalStatus::Completed {
                    obstacles.push(DreamObstacle {
                        description: format!("Dependency '{}' not complete", dep.title),
                        severity: 0.7,
                        timing: "Before completion".to_string(),
                        mitigation: Some(format!("Complete {} first", dep.title)),
                    });
                }
            }
        }

        obstacles
    }

    pub(crate) fn extract_insights(
        &self,
        goal: &Goal,
        scenario: &CompletionScenario,
        obstacles: &[DreamObstacle],
    ) -> Vec<DreamInsight> {
        let mut insights = vec![DreamInsight {
            insight: format!("Primary completion signal: {}", scenario.feeling),
            actionable: true,
            action: Some("Prioritize highest leverage task this week".to_string()),
        }];

        if !obstacles.is_empty() {
            insights.push(DreamInsight {
                insight: format!("{} blockers predicted", obstacles.len()),
                actionable: true,
                action: Some("Resolve blockers before deep execution".to_string()),
            });
        }

        if goal.progress.percentage < 0.2 {
            insights.push(DreamInsight {
                insight: "Early-stage goal: momentum building is critical".to_string(),
                actionable: true,
                action: Some("Ship a visible milestone in 7 days".to_string()),
            });
        }

        insights
    }

    pub(crate) fn discover_sub_goals(
        &self,
        goal: &Goal,
        obstacles: &[DreamObstacle],
    ) -> Vec<GoalSeed> {
        obstacles
            .iter()
            .take(2)
            .enumerate()
            .map(|(i, o)| GoalSeed {
                title: format!("Mitigate blocker {}", i + 1),
                description: o.description.clone(),
                parent: goal.id,
                reason: "Dream-derived mitigation".to_string(),
            })
            .collect()
    }

    pub(crate) fn calculate_dream_confidence(&self, goal: &Goal) -> f64 {
        let blocker_penalty = goal
            .blockers
            .iter()
            .filter(|b| b.resolved_at.is_none())
            .map(|b| b.severity)
            .sum::<f64>()
            .min(1.0);
        (0.8 - blocker_penalty * 0.5 + goal.progress.percentage * 0.2).clamp(0.1, 1.0)
    }
}
