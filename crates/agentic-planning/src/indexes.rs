use crate::types::{
    Commitment, CommitmentId, CommitmentStatus, Decision, DecisionId, DecisionStatus, Dream,
    DreamId, Federation, FederationId, Goal, GoalId, GoalRelationship, GoalStatus, Priority,
    StakeholderId, SyncStatus, Timestamp, UrgentItem, UrgentItemType,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanIndexes {
    pub goals_by_status: HashMap<GoalStatus, Vec<GoalId>>,
    pub goals_by_priority: HashMap<Priority, Vec<GoalId>>,
    pub goals_by_deadline: BTreeMap<Timestamp, Vec<GoalId>>,
    pub goals_by_parent: HashMap<GoalId, Vec<GoalId>>,
    pub root_goals: Vec<GoalId>,
    pub active_goals: Vec<GoalId>,
    pub blocked_goals: Vec<GoalId>,
    pub goals_by_tag: HashMap<String, Vec<GoalId>>,
    pub decisions_by_goal: HashMap<GoalId, Vec<DecisionId>>,
    pub decisions_by_time: BTreeMap<Timestamp, Vec<DecisionId>>,
    pub pending_decisions: Vec<DecisionId>,
    pub regretted_decisions: Vec<DecisionId>,
    pub decision_chains: HashMap<DecisionId, Vec<DecisionId>>,
    pub commitments_by_due: BTreeMap<Timestamp, Vec<CommitmentId>>,
    pub commitments_by_stakeholder: HashMap<StakeholderId, Vec<CommitmentId>>,
    pub commitments_by_goal: HashMap<GoalId, Vec<CommitmentId>>,
    pub active_commitments: Vec<CommitmentId>,
    pub at_risk_commitments: Vec<CommitmentId>,
    pub urgent_items: Vec<UrgentItem>,
    pub goal_relationships: HashMap<GoalId, Vec<(GoalId, GoalRelationship)>>,
    pub commitment_entanglements: HashMap<CommitmentId, Vec<CommitmentId>>,
    pub dreams_by_goal: HashMap<GoalId, Vec<DreamId>>,
    pub federations_by_member: HashMap<String, Vec<FederationId>>,
    pub federations_by_sync: HashMap<SyncStatus, Vec<FederationId>>,
}

impl PlanIndexes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_goal(&mut self, goal: &Goal) {
        self.goals_by_status
            .entry(goal.status)
            .or_default()
            .push(goal.id);

        self.goals_by_priority
            .entry(goal.priority)
            .or_default()
            .push(goal.id);

        if let Some(deadline) = goal.deadline {
            self.goals_by_deadline
                .entry(deadline)
                .or_default()
                .push(goal.id);
        }

        if let Some(parent) = goal.parent {
            self.goals_by_parent
                .entry(parent)
                .or_default()
                .push(goal.id);
        } else {
            self.root_goals.push(goal.id);
        }

        for tag in &goal.tags {
            self.goals_by_tag
                .entry(tag.clone())
                .or_default()
                .push(goal.id);
        }

        if goal.status == GoalStatus::Active {
            self.active_goals.push(goal.id);
        }
        if goal.status == GoalStatus::Blocked {
            self.blocked_goals.push(goal.id);
        }

        // R1: Index goal relationships (dependencies, parent-child, explicit relationships)
        for dep_id in &goal.dependencies {
            let rel = GoalRelationship::Dependency {
                dependent: goal.id,
                on: *dep_id,
                strength: 1.0,
            };
            self.goal_relationships
                .entry(goal.id)
                .or_default()
                .push((*dep_id, rel.clone()));
            self.goal_relationships
                .entry(*dep_id)
                .or_default()
                .push((goal.id, rel));
        }
        if let Some(parent) = goal.parent {
            let rel = GoalRelationship::ParentChild {
                parent,
                child: goal.id,
            };
            self.goal_relationships
                .entry(goal.id)
                .or_default()
                .push((parent, rel.clone()));
            self.goal_relationships
                .entry(parent)
                .or_default()
                .push((goal.id, rel));
        }
        for explicit_rel in &goal.relationships {
            match explicit_rel {
                GoalRelationship::Alliance { goals, .. }
                | GoalRelationship::Rivalry { goals, .. }
                | GoalRelationship::Romance { goals, .. } => {
                    let other = if goals.0 == goal.id { goals.1 } else { goals.0 };
                    self.goal_relationships
                        .entry(goal.id)
                        .or_default()
                        .push((other, explicit_rel.clone()));
                }
                _ => {}
            }
        }

        self.check_urgent_goal(goal);
    }

    pub fn goal_status_changed(&mut self, id: GoalId, old: GoalStatus, new: GoalStatus) {
        if let Some(list) = self.goals_by_status.get_mut(&old) {
            list.retain(|x| *x != id);
        }
        self.goals_by_status.entry(new).or_default().push(id);

        if old == GoalStatus::Active {
            self.active_goals.retain(|x| *x != id);
        }
        if old == GoalStatus::Blocked {
            self.blocked_goals.retain(|x| *x != id);
        }

        if new == GoalStatus::Active {
            self.active_goals.push(id);
        }
        if new == GoalStatus::Blocked {
            self.blocked_goals.push(id);
        }
    }

    pub fn goal_priority_changed(&mut self, id: GoalId, old: Priority, new: Priority) {
        if let Some(list) = self.goals_by_priority.get_mut(&old) {
            list.retain(|x| *x != id);
        }
        self.goals_by_priority.entry(new).or_default().push(id);
    }

    pub fn goal_activated(&mut self, id: GoalId) {
        self.goal_status_changed(id, GoalStatus::Draft, GoalStatus::Active);
    }

    pub fn goal_completed(&mut self, id: GoalId) {
        self.active_goals.retain(|x| *x != id);
        self.blocked_goals.retain(|x| *x != id);
    }

    pub fn goal_abandoned(&mut self, id: GoalId) {
        self.active_goals.retain(|x| *x != id);
        self.blocked_goals.retain(|x| *x != id);
    }

    pub fn goal_blocked(&mut self, id: GoalId) {
        if !self.blocked_goals.contains(&id) {
            self.blocked_goals.push(id);
        }
        self.active_goals.retain(|x| *x != id);
        // Keep goals_by_status consistent
        if let Some(list) = self.goals_by_status.get_mut(&GoalStatus::Active) {
            list.retain(|x| *x != id);
        }
        self.goals_by_status
            .entry(GoalStatus::Blocked)
            .or_default()
            .push(id);
    }

    pub fn goal_unblocked(&mut self, id: GoalId) {
        self.blocked_goals.retain(|x| *x != id);
        if !self.active_goals.contains(&id) {
            self.active_goals.push(id);
        }
        // Keep goals_by_status consistent
        if let Some(list) = self.goals_by_status.get_mut(&GoalStatus::Blocked) {
            list.retain(|x| *x != id);
        }
        self.goals_by_status
            .entry(GoalStatus::Active)
            .or_default()
            .push(id);
    }

    pub fn goal_linked(&mut self, relationship: &GoalRelationship) {
        match relationship {
            GoalRelationship::Dependency { dependent, on, .. } => {
                let rel = relationship.clone();
                self.goal_relationships
                    .entry(*dependent)
                    .or_default()
                    .push((*on, rel.clone()));
                self.goal_relationships
                    .entry(*on)
                    .or_default()
                    .push((*dependent, rel));
            }
            GoalRelationship::ParentChild { parent, child } => {
                let rel = relationship.clone();
                self.goal_relationships
                    .entry(*child)
                    .or_default()
                    .push((*parent, rel.clone()));
                self.goal_relationships
                    .entry(*parent)
                    .or_default()
                    .push((*child, rel));
            }
            GoalRelationship::Alliance { goals, .. }
            | GoalRelationship::Rivalry { goals, .. }
            | GoalRelationship::Romance { goals, .. }
            | GoalRelationship::Nemesis { goals, .. } => {
                self.goal_relationships
                    .entry(goals.0)
                    .or_default()
                    .push((goals.1, relationship.clone()));
                self.goal_relationships
                    .entry(goals.1)
                    .or_default()
                    .push((goals.0, relationship.clone()));
            }
            GoalRelationship::Successor {
                predecessor,
                successor,
            } => {
                let rel = relationship.clone();
                self.goal_relationships
                    .entry(*predecessor)
                    .or_default()
                    .push((*successor, rel.clone()));
                self.goal_relationships
                    .entry(*successor)
                    .or_default()
                    .push((*predecessor, rel));
            }
        }
    }

    pub fn add_decision(&mut self, decision: &Decision) {
        for goal_id in &decision.affected_goals {
            self.decisions_by_goal
                .entry(*goal_id)
                .or_default()
                .push(decision.id);
        }

        if let Some(at) = decision.crystallized_at {
            self.decisions_by_time
                .entry(at)
                .or_default()
                .push(decision.id);
        }

        if decision.status == DecisionStatus::Pending {
            self.pending_decisions.push(decision.id);
        }
    }

    pub fn decision_crystallized(&mut self, id: DecisionId) {
        self.pending_decisions.retain(|x| *x != id);
    }

    pub fn add_commitment(&mut self, commitment: &Commitment) {
        if let Some(due) = commitment.due {
            self.commitments_by_due
                .entry(due)
                .or_default()
                .push(commitment.id);
        }

        self.commitments_by_stakeholder
            .entry(commitment.made_to.id)
            .or_default()
            .push(commitment.id);

        if let Some(goal_id) = commitment.goal {
            self.commitments_by_goal
                .entry(goal_id)
                .or_default()
                .push(commitment.id);
        }

        if commitment.status == CommitmentStatus::Active {
            self.active_commitments.push(commitment.id);
        }

        // R2: Index commitment entanglements (bidirectional)
        for entanglement in &commitment.entanglements {
            self.commitment_entanglements
                .entry(commitment.id)
                .or_default()
                .push(entanglement.with);
            self.commitment_entanglements
                .entry(entanglement.with)
                .or_default()
                .push(commitment.id);
        }

        self.check_urgent_commitment(commitment);
    }

    pub fn commitment_fulfilled(&mut self, id: CommitmentId) {
        self.active_commitments.retain(|x| *x != id);
        self.at_risk_commitments.retain(|x| *x != id);
    }

    pub fn commitment_broken(&mut self, id: CommitmentId) {
        self.active_commitments.retain(|x| *x != id);
    }

    /// R1: Clean up goal relationship entries when a goal is removed
    pub fn remove_goal(&mut self, id: GoalId) {
        self.goals_by_status
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.goals_by_priority
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.goals_by_deadline
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.goals_by_parent
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.goals_by_parent.remove(&id);
        self.root_goals.retain(|x| *x != id);
        self.active_goals.retain(|x| *x != id);
        self.blocked_goals.retain(|x| *x != id);
        self.goals_by_tag
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        // Clean up relationship index: remove own entry and references from others
        self.goal_relationships.remove(&id);
        for rels in self.goal_relationships.values_mut() {
            rels.retain(|(other, _)| *other != id);
        }
        self.urgent_items.retain(|u| u.id != id.0);
    }

    /// R2: Clean up commitment entanglement entries when a commitment is removed
    pub fn remove_commitment(&mut self, id: CommitmentId) {
        self.commitments_by_due
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.commitments_by_stakeholder
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.commitments_by_goal
            .values_mut()
            .for_each(|v| v.retain(|x| *x != id));
        self.active_commitments.retain(|x| *x != id);
        self.at_risk_commitments.retain(|x| *x != id);
        // Clean up entanglement index: remove own entry and references from others
        self.commitment_entanglements.remove(&id);
        for linked in self.commitment_entanglements.values_mut() {
            linked.retain(|x| *x != id);
        }
        self.urgent_items.retain(|u| u.id != id.0);
    }

    /// R3: Index a dream by its goal_id
    pub fn add_dream(&mut self, dream: &Dream) {
        self.dreams_by_goal
            .entry(dream.goal_id)
            .or_default()
            .push(dream.id);
    }

    /// R3: Index a federation by member agent_ids and sync_status
    pub fn add_federation(&mut self, federation: &Federation) {
        for member in &federation.members {
            self.federations_by_member
                .entry(member.agent_id.clone())
                .or_default()
                .push(federation.id);
        }
        self.federations_by_sync
            .entry(federation.sync_status)
            .or_default()
            .push(federation.id);
    }

    fn check_urgent_goal(&mut self, goal: &Goal) {
        if let Some(deadline) = goal.deadline {
            let now = Timestamp::now();
            let days = (deadline.0 - now.0) as f64 / (86_400.0 * 1e9);
            if days <= 7.0 && goal.status == GoalStatus::Active {
                self.urgent_items.push(UrgentItem {
                    item_type: UrgentItemType::Goal,
                    id: goal.id.0,
                    deadline,
                    urgency: goal.feelings.urgency,
                });
            }
        }
    }

    fn check_urgent_commitment(&mut self, commitment: &Commitment) {
        if let Some(due) = commitment.due {
            let now = Timestamp::now();
            let days = (due.0 - now.0) as f64 / (86_400.0 * 1e9);
            if days <= 7.0 && commitment.status == CommitmentStatus::Active {
                self.urgent_items.push(UrgentItem {
                    item_type: UrgentItemType::Commitment,
                    id: commitment.id.0,
                    deadline: due,
                    urgency: commitment.weight,
                });
            }
        }
    }

    pub fn rebuild(
        &mut self,
        goals: &HashMap<GoalId, Goal>,
        decisions: &HashMap<DecisionId, Decision>,
        commitments: &HashMap<CommitmentId, Commitment>,
    ) {
        self.rebuild_full(
            goals,
            decisions,
            commitments,
            &HashMap::new(),
            &HashMap::new(),
        );
    }

    /// R3: Full rebuild including dreams and federations
    pub fn rebuild_full(
        &mut self,
        goals: &HashMap<GoalId, Goal>,
        decisions: &HashMap<DecisionId, Decision>,
        commitments: &HashMap<CommitmentId, Commitment>,
        dreams: &HashMap<DreamId, Dream>,
        federations: &HashMap<FederationId, Federation>,
    ) {
        *self = Self::new();

        for goal in goals.values() {
            self.add_goal(goal);
        }

        for decision in decisions.values() {
            self.add_decision(decision);
        }

        for commitment in commitments.values() {
            self.add_commitment(commitment);
        }

        for dream in dreams.values() {
            self.add_dream(dream);
        }

        for federation in federations.values() {
            self.add_federation(federation);
        }
    }
}
