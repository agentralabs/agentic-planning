use crate::types::*;
use crate::{Error, PlanningEngine, Result};
use uuid::Uuid;

impl PlanningEngine {
    pub fn create_goal(&mut self, request: CreateGoalRequest) -> Result<Goal> {
        let id = GoalId(Uuid::new_v4());
        let now = Timestamp::now();
        let urgency = self.calculate_initial_urgency(&request);
        let gravity = self.calculate_initial_gravity(&request);
        let inertia = self.calculate_initial_inertia(&request);

        let soul = GoalSoul {
            intention: request.intention.clone(),
            significance: request.significance.unwrap_or_default(),
            success_criteria: request.success_criteria.unwrap_or_default(),
            emotional_weight: request.emotional_weight.unwrap_or(0.5),
            values: request.values.unwrap_or_default(),
        };

        let feelings = GoalFeelings {
            urgency,
            neglect: 0.0,
            confidence: 0.5,
            alignment: 1.0,
            vitality: 1.0,
            last_calculated: now,
        };

        let physics = GoalPhysics {
            momentum: 0.0,
            gravity,
            inertia,
            energy: 1.0,
            last_calculated: now,
        };

        let goal = Goal {
            id,
            title: request.title,
            description: request.description,
            soul,
            status: GoalStatus::Draft,
            created_at: now,
            activated_at: None,
            completed_at: None,
            deadline: request.deadline,
            parent: request.parent,
            children: Vec::new(),
            dependencies: request.dependencies.unwrap_or_default(),
            dependents: Vec::new(),
            relationships: Vec::new(),
            priority: request.priority.unwrap_or(Priority::Medium),
            progress: Progress::new(),
            feelings,
            physics,
            blockers: Vec::new(),
            decisions: Vec::new(),
            commitments: Vec::new(),
            dreams: Vec::new(),
            tags: request.tags.unwrap_or_default(),
            metadata: request.metadata.unwrap_or_default(),
            provenance: GoalProvenance {
                origin: request.origin.unwrap_or(ProvenanceOrigin::UserRequest),
                user_request: request.user_request,
                session_id: request.session_id,
                creation_context: request.context.unwrap_or_default(),
            },
            metamorphosis: None,
            previous_life: None,
        };

        self.goal_store.insert(id, goal.clone());

        if let Some(parent_id) = request.parent {
            if let Some(parent) = self.goal_store.get_mut(&parent_id) {
                parent.children.push(id);
            }
        }

        for dep_id in &goal.dependencies {
            if let Some(dep) = self.goal_store.get_mut(dep_id) {
                dep.dependents.push(id);
            }
        }

        self.indexes.add_goal(&goal);
        self.mark_dirty();
        Ok(goal)
    }

    pub fn activate_goal(&mut self, id: GoalId) -> Result<Goal> {
        let out = {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            if goal.status != GoalStatus::Draft && goal.status != GoalStatus::Reborn {
                return Err(Error::InvalidTransition {
                    from: goal.status,
                    to: GoalStatus::Active,
                });
            }

            let old = goal.status;
            goal.status = GoalStatus::Active;
            goal.activated_at = Some(Timestamp::now());
            goal.feelings.vitality = 1.0;
            self.indexes
                .goal_status_changed(id, old, GoalStatus::Active);
            goal.clone()
        };
        self.mark_dirty();
        Ok(out)
    }

    pub fn pause_goal(&mut self, id: GoalId, reason: Option<String>) -> Result<Goal> {
        let out = {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            let old = goal.status;
            goal.status = GoalStatus::Paused;
            if let Some(r) = reason {
                goal.metadata
                    .insert("pause_reason".to_string(), serde_json::Value::String(r));
                goal.metadata.insert(
                    "paused_at".to_string(),
                    serde_json::Value::String(Timestamp::now().0.to_string()),
                );
            }
            self.indexes
                .goal_status_changed(id, old, GoalStatus::Paused);
            goal.clone()
        };
        self.mark_dirty();
        Ok(out)
    }

    pub fn resume_goal(&mut self, id: GoalId) -> Result<Goal> {
        let out = {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            let old = goal.status;
            goal.status = GoalStatus::Active;
            self.indexes
                .goal_status_changed(id, old, GoalStatus::Active);
            goal.clone()
        };
        self.mark_dirty();
        Ok(out)
    }

    pub fn progress_goal(
        &mut self,
        id: GoalId,
        percentage: f64,
        note: Option<String>,
    ) -> Result<Goal> {
        let now = Timestamp::now();
        let snapshot = {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            let p = percentage.clamp(0.0, 1.0);
            goal.progress.history.push(ProgressPoint {
                timestamp: now,
                percentage: p,
                note,
            });
            goal.progress.percentage = p;
            goal.clone()
        };

        let velocity = self.calculate_velocity(&snapshot.progress.history);
        let momentum = self.calculate_momentum_from_goal(&snapshot);
        let confidence = self.calculate_confidence_from_goal(&snapshot);

        {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            goal.progress.velocity = velocity;
            if velocity > 0.0 {
                let remaining = 1.0 - goal.progress.percentage;
                let days_remaining = remaining / velocity;
                goal.progress.eta = Some(Timestamp::days_from_now(days_remaining));
            }
            goal.physics.momentum = momentum;
            goal.feelings.neglect = 0.0;
            goal.feelings.confidence = confidence;
            goal.feelings.last_calculated = now;

            if goal.progress.percentage >= 1.0 {
                for c in &mut goal.soul.success_criteria {
                    c.achieved = true;
                    c.achieved_at = Some(now);
                }
            }
        }

        self.mark_dirty();
        Ok(self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone())
    }

    pub fn complete_goal(&mut self, id: GoalId, note: Option<String>) -> Result<Goal> {
        let now = Timestamp::now();
        {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            if !matches!(
                goal.status,
                GoalStatus::Active | GoalStatus::Blocked | GoalStatus::Paused
            ) {
                return Err(Error::CannotComplete(goal.status));
            }
            let old = goal.status;
            goal.status = GoalStatus::Completed;
            goal.completed_at = Some(now);
            goal.progress.percentage = 1.0;
            goal.progress.history.push(ProgressPoint {
                timestamp: now,
                percentage: 1.0,
                note,
            });
            self.indexes
                .goal_status_changed(id, old, GoalStatus::Completed);
        }

        // Archive the goal's soul for potential reincarnation
        let goal = self.goal_store.get(&id).ok_or(Error::GoalNotFound(id))?;
        let karma = GoalKarma {
            failures: Vec::new(),
            near_successes: Vec::new(),
            requirements_for_success: goal
                .soul
                .success_criteria
                .iter()
                .filter(|c| c.achieved)
                .map(|c| c.description.clone())
                .collect(),
            invested_energy: goal.physics.energy,
        };
        let reincarnation_potential = self.calculate_reincarnation_potential(goal);
        let soul_archive = GoalSoulArchive {
            original_id: id,
            soul: goal.soul.clone(),
            death_record: GoalDeath {
                cause: "completed".to_string(),
                timestamp: now,
            },
            karma,
            reincarnation_potential,
            trigger_conditions: vec!["similar goal created".to_string()],
        };
        self.soul_archive.insert(id, soul_archive);

        self.release_completion_energy(id);
        let dependents = self
            .goal_store
            .get(&id)
            .map(|g| g.dependents.clone())
            .unwrap_or_default();
        for dep in dependents {
            self.check_unblock(&dep);
        }

        self.mark_dirty();
        Ok(self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone())
    }

    pub fn abandon_goal(&mut self, id: GoalId, reason: String) -> Result<Goal> {
        let now = Timestamp::now();
        let snapshot = self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone();

        let karma = GoalKarma {
            failures: vec![reason.clone()],
            near_successes: Vec::new(),
            requirements_for_success: Vec::new(),
            invested_energy: snapshot.progress.percentage,
        };

        self.soul_archive.insert(
            id,
            GoalSoulArchive {
                original_id: id,
                soul: snapshot.soul.clone(),
                death_record: GoalDeath {
                    cause: reason,
                    timestamp: now,
                },
                karma,
                reincarnation_potential: self.calculate_reincarnation_potential(&snapshot),
                trigger_conditions: Vec::new(),
            },
        );

        if let Some(goal) = self.goal_store.get_mut(&id) {
            let old = goal.status;
            goal.status = GoalStatus::Abandoned;
            goal.completed_at = Some(now);
            goal.physics.momentum = 0.0;
            goal.physics.energy = 0.0;
            self.indexes
                .goal_status_changed(id, old, GoalStatus::Abandoned);
        }

        self.mark_dirty();
        Ok(self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone())
    }

    pub fn reincarnate_goal(
        &mut self,
        original_id: GoalId,
        updates: ReincarnationUpdates,
    ) -> Result<Goal> {
        let archive = self
            .soul_archive
            .get(&original_id)
            .ok_or(Error::SoulNotFound(original_id))?
            .clone();

        let request = CreateGoalRequest {
            title: updates
                .title
                .unwrap_or_else(|| format!("{} (Reborn)", archive.soul.intention)),
            description: updates.description.unwrap_or_default(),
            intention: archive.soul.intention.clone(),
            significance: Some(archive.soul.significance.clone()),
            success_criteria: Some(archive.soul.success_criteria.clone()),
            emotional_weight: Some(archive.soul.emotional_weight),
            values: Some(archive.soul.values.clone()),
            origin: Some(ProvenanceOrigin::Reincarnation {
                previous: original_id,
            }),
            ..Default::default()
        };

        let mut goal = self.create_goal(request)?;
        goal.previous_life = Some(PreviousLife {
            original_id,
            death_cause: archive.death_record.cause,
            lessons_learned: updates.lessons_learned.unwrap_or_default(),
            karma: archive.karma,
        });
        goal.status = GoalStatus::Reborn;
        self.goal_store.insert(goal.id, goal.clone());

        self.mark_dirty();
        Ok(goal)
    }

    pub fn decompose_goal(
        &mut self,
        id: GoalId,
        sub_goals: Vec<CreateGoalRequest>,
    ) -> Result<Vec<Goal>> {
        let _ = self.goal_store.get(&id).ok_or(Error::GoalNotFound(id))?;
        let mut created = Vec::new();

        for mut request in sub_goals {
            request.parent = Some(id);
            request.origin = Some(ProvenanceOrigin::Decomposition { parent: id });
            let child = self.create_goal(request)?;
            created.push(child);
        }

        Ok(created)
    }

    pub fn block_goal(&mut self, id: GoalId, blocker: Blocker) -> Result<Goal> {
        let mut snapshot = self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone();
        snapshot.blockers.push(blocker);
        snapshot.feelings.confidence = self.calculate_confidence_from_goal(&snapshot);

        if let Some(goal) = self.goal_store.get_mut(&id) {
            goal.blockers = snapshot.blockers;
            goal.feelings.confidence = snapshot.feelings.confidence;
            if goal.status == GoalStatus::Active {
                goal.status = GoalStatus::Blocked;
                self.indexes.goal_blocked(id);
            }
        }

        self.mark_dirty();
        Ok(self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone())
    }

    pub fn unblock_goal(
        &mut self,
        id: GoalId,
        blocker_id: Uuid,
        resolution: String,
    ) -> Result<Goal> {
        let now = Timestamp::now();
        {
            let goal = self
                .goal_store
                .get_mut(&id)
                .ok_or(Error::GoalNotFound(id))?;
            for blocker in &mut goal.blockers {
                if blocker.id == blocker_id {
                    blocker.resolved_at = Some(now);
                    blocker.resolution = Some(resolution.clone());
                }
            }

            let all_resolved = goal.blockers.iter().all(|b| b.resolved_at.is_some());
            if all_resolved && goal.status == GoalStatus::Blocked {
                goal.status = GoalStatus::Active;
                self.indexes.goal_unblocked(id);
            }
        }

        let snapshot = self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone();
        let confidence = self.calculate_confidence_from_goal(&snapshot);
        if let Some(goal) = self.goal_store.get_mut(&id) {
            goal.feelings.confidence = confidence;
        }

        self.mark_dirty();
        Ok(self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone())
    }

    pub fn link_goals(&mut self, relationship: GoalRelationship) -> Result<()> {
        match &relationship {
            GoalRelationship::Alliance { goals: (a, b), .. }
            | GoalRelationship::Rivalry { goals: (a, b), .. }
            | GoalRelationship::Romance { goals: (a, b), .. }
            | GoalRelationship::Nemesis { goals: (a, b), .. } => {
                if let Some(goal_a) = self.goal_store.get_mut(a) {
                    goal_a.relationships.push(relationship.clone());
                }
                if let Some(goal_b) = self.goal_store.get_mut(b) {
                    goal_b.relationships.push(relationship.clone());
                }
            }
            GoalRelationship::Dependency { dependent, on, .. } => {
                if let Some(dep) = self.goal_store.get_mut(dependent) {
                    dep.dependencies.push(*on);
                }
                if let Some(target) = self.goal_store.get_mut(on) {
                    target.dependents.push(*dependent);
                }
            }
            GoalRelationship::ParentChild { parent, child } => {
                if let Some(parent_goal) = self.goal_store.get_mut(parent) {
                    parent_goal.children.push(*child);
                }
                if let Some(child_goal) = self.goal_store.get_mut(child) {
                    child_goal.parent = Some(*parent);
                }
            }
            GoalRelationship::Successor {
                predecessor,
                successor,
            } => {
                if let Some(p) = self.goal_store.get_mut(predecessor) {
                    p.relationships.push(relationship.clone());
                }
                if let Some(s) = self.goal_store.get_mut(successor) {
                    s.relationships.push(relationship.clone());
                }
            }
        }

        self.indexes.goal_linked(&relationship);
        self.mark_dirty();
        Ok(())
    }

    pub fn create_decision(&mut self, request: CreateDecisionRequest) -> Result<Decision> {
        let id = DecisionId(Uuid::new_v4());
        let now = Timestamp::now();

        let decision = Decision {
            id,
            question: DecisionQuestion {
                question: request.question,
                context: request.context.unwrap_or_default(),
                constraints: request.constraints.unwrap_or_default(),
                asked_at: now,
            },
            status: DecisionStatus::Pending,
            crystallized_at: None,
            chosen: None,
            shadows: Vec::new(),
            reasoning: DecisionReasoning::default(),
            decider: request.decider.unwrap_or(Decider::User { name: None }),
            affected_goals: request.goals.unwrap_or_default(),
            caused_by: request.caused_by,
            causes: Vec::new(),
            reversibility: Reversibility::default(),
            consequences: Vec::new(),
            regret_score: 0.0,
            regret_updated_at: None,
        };

        for goal_id in &decision.affected_goals {
            if let Some(goal) = self.goal_store.get_mut(goal_id) {
                goal.decisions.push(id);
            }
        }

        if let Some(parent_id) = request.caused_by {
            if let Some(parent) = self.decision_store.get_mut(&parent_id) {
                parent.causes.push(id);
            }
        }

        self.decision_store.insert(id, decision.clone());
        self.indexes.add_decision(&decision);
        self.mark_dirty();
        Ok(decision)
    }

    pub fn add_option(&mut self, id: DecisionId, path: DecisionPath) -> Result<Decision> {
        let out = {
            let decision = self
                .decision_store
                .get_mut(&id)
                .ok_or(Error::DecisionNotFound(id))?;

            if decision.status == DecisionStatus::Crystallized {
                return Err(Error::AlreadyCrystallized);
            }

            decision.status = DecisionStatus::Deliberating;
            decision.shadows.push(CrystalShadow {
                path,
                rejection_reason: String::new(),
                counterfactual: None,
                resurrection_cost: 0.0,
            });
            decision.clone()
        };

        self.mark_dirty();
        Ok(out)
    }

    pub fn crystallize(
        &mut self,
        id: DecisionId,
        chosen_path_id: PathId,
        reasoning: DecisionReasoning,
    ) -> Result<Decision> {
        let now = Timestamp::now();
        let snapshot = self
            .decision_store
            .get(&id)
            .ok_or(Error::DecisionNotFound(id))?
            .clone();

        if snapshot.status == DecisionStatus::Crystallized {
            return Err(Error::AlreadyCrystallized);
        }

        let chosen_idx = snapshot
            .shadows
            .iter()
            .position(|s| s.path.id == chosen_path_id)
            .ok_or(Error::PathNotFound(chosen_path_id))?;

        let mut chosen_shadow = snapshot.shadows[chosen_idx].clone();
        let mut new_shadows = Vec::new();

        for (idx, shadow) in snapshot.shadows.iter().enumerate() {
            if idx == chosen_idx {
                continue;
            }
            let mut s = shadow.clone();
            if s.rejection_reason.is_empty() {
                s.rejection_reason = "Not chosen".to_string();
            }
            s.resurrection_cost = self.calculate_resurrection_cost(&snapshot, &s);
            new_shadows.push(s);
        }

        let reversibility = self.calculate_reversibility(&snapshot);

        if let Some(decision) = self.decision_store.get_mut(&id) {
            chosen_shadow.resurrection_cost = 0.0;
            decision.chosen = Some(chosen_shadow.path);
            decision.shadows = new_shadows;
            decision.status = DecisionStatus::Crystallized;
            decision.crystallized_at = Some(now);
            decision.reasoning = reasoning;
            decision.reversibility = reversibility;
        }

        self.indexes.decision_crystallized(id);
        self.mark_dirty();
        Ok(self
            .decision_store
            .get(&id)
            .ok_or(Error::DecisionNotFound(id))?
            .clone())
    }

    pub fn record_consequence(
        &mut self,
        id: DecisionId,
        consequence: Consequence,
    ) -> Result<Decision> {
        let snapshot = self
            .decision_store
            .get(&id)
            .ok_or(Error::DecisionNotFound(id))?
            .clone();

        let mut updated = snapshot.clone();
        updated.consequences.push(consequence);
        updated.regret_score = self.calculate_regret(&updated);
        updated.regret_updated_at = Some(Timestamp::now());
        if updated.regret_score > 0.7 {
            updated.status = DecisionStatus::Regretted;
        }

        self.decision_store.insert(id, updated.clone());
        self.mark_dirty();
        Ok(updated)
    }

    pub fn recrystallize(
        &mut self,
        id: DecisionId,
        new_path_id: PathId,
        reason: String,
    ) -> Result<Decision> {
        let mut decision = self
            .decision_store
            .get(&id)
            .ok_or(Error::DecisionNotFound(id))?
            .clone();

        if decision.status != DecisionStatus::Crystallized
            && decision.status != DecisionStatus::Regretted
        {
            return Err(Error::CannotRecrystallize(decision.status));
        }

        if let Some(old_chosen) = decision.chosen.take() {
            decision.shadows.push(CrystalShadow {
                path: old_chosen,
                rejection_reason: format!("Recrystallized: {}", reason),
                counterfactual: None,
                resurrection_cost: 0.0,
            });
        }

        let new_idx = decision
            .shadows
            .iter()
            .position(|s| s.path.id == new_path_id)
            .ok_or(Error::PathNotFound(new_path_id))?;

        let new_chosen = decision.shadows.remove(new_idx);
        decision.chosen = Some(new_chosen.path);
        decision.status = DecisionStatus::Recrystallized;
        decision.crystallized_at = Some(Timestamp::now());
        decision.regret_score = 0.0;
        decision.reasoning.rationale = format!(
            "Recrystallized: {}. Previous: {}",
            reason, decision.reasoning.rationale
        );
        decision
            .reasoning
            .factors_considered
            .push(format!("Recrystallization reason: {}", reason));

        self.decision_store.insert(id, decision.clone());
        self.mark_dirty();
        Ok(decision)
    }

    pub fn create_commitment(&mut self, request: CreateCommitmentRequest) -> Result<Commitment> {
        let id = CommitmentId(Uuid::new_v4());
        let now = Timestamp::now();
        let weight = self.calculate_commitment_weight(&request);
        let inertia = self.calculate_commitment_inertia(&request);
        let breaking_cost = self.calculate_breaking_cost(&request);

        let commitment = Commitment {
            id,
            promise: request.promise,
            made_to: request.stakeholder,
            made_at: now,
            due: request.due,
            status: CommitmentStatus::Active,
            weight,
            inertia,
            breaking_cost,
            goal: request.goal,
            entanglements: Vec::new(),
            fulfillment: None,
            renegotiations: Vec::new(),
        };

        if let Some(goal_id) = request.goal {
            if let Some(goal) = self.goal_store.get_mut(&goal_id) {
                goal.commitments.push(id);
            }
        }

        self.commitment_store.insert(id, commitment.clone());
        self.indexes.add_commitment(&commitment);
        self.mark_dirty();
        Ok(commitment)
    }

    pub fn fulfill_commitment(
        &mut self,
        id: CommitmentId,
        how_delivered: String,
    ) -> Result<Commitment> {
        let now = Timestamp::now();

        let chain_bonus = self.calculate_chain_bonus(id);
        let (status, weight, entanglements) = {
            let c = self
                .commitment_store
                .get(&id)
                .ok_or(Error::CommitmentNotFound(id))?;
            (c.status, c.weight, c.entanglements.clone())
        };

        if status != CommitmentStatus::Active {
            return Err(Error::CannotFulfill(status));
        }

        let energy_released = weight * (1.0 + chain_bonus);
        if let Some(c) = self.commitment_store.get_mut(&id) {
            c.status = CommitmentStatus::Fulfilled;
            c.fulfillment = Some(CommitmentFulfillment {
                fulfilled_at: now,
                how_delivered,
                energy_released,
                trust_gained: c.weight * 0.5,
            });
        }

        for entanglement in &entanglements {
            if entanglement.entanglement_type == EntanglementType::Sequential {
                let _ = self.boost_commitment(entanglement.with, energy_released * 0.3);
            }
        }

        self.indexes.commitment_fulfilled(id);
        self.mark_dirty();
        Ok(self
            .commitment_store
            .get(&id)
            .ok_or(Error::CommitmentNotFound(id))?
            .clone())
    }

    pub fn break_commitment(&mut self, id: CommitmentId, reason: String) -> Result<Commitment> {
        let commitment_ref = self
            .commitment_store
            .get(&id)
            .ok_or(Error::CommitmentNotFound(id))?;
        if matches!(
            commitment_ref.status,
            CommitmentStatus::Fulfilled | CommitmentStatus::Broken
        ) {
            return Err(Error::CannotBreak(commitment_ref.status));
        }
        let entanglements = commitment_ref.entanglements.clone();

        if let Some(commitment) = self.commitment_store.get_mut(&id) {
            commitment.status = CommitmentStatus::Broken;
            // Store the break reason as a renegotiation record with accepted=false
            commitment.renegotiations.push(Renegotiation {
                renegotiated_at: Timestamp::now(),
                original: commitment.promise.clone(),
                new: Promise::default(),
                reason,
                accepted: false,
                trust_impact: -commitment.breaking_cost.trust_damage,
            });
        }

        for entanglement in entanglements {
            if entanglement.entanglement_type == EntanglementType::Parallel {
                let _ = self.destabilize_commitment(entanglement.with);
            }
        }

        self.indexes.commitment_broken(id);
        self.mark_dirty();
        Ok(self
            .commitment_store
            .get(&id)
            .ok_or(Error::CommitmentNotFound(id))?
            .clone())
    }

    pub fn renegotiate_commitment(
        &mut self,
        id: CommitmentId,
        new_promise: Promise,
        reason: String,
    ) -> Result<Commitment> {
        let now = Timestamp::now();
        let out = {
            let commitment = self
                .commitment_store
                .get_mut(&id)
                .ok_or(Error::CommitmentNotFound(id))?;

            let old_promise = commitment.promise.clone();

            // Assess renegotiation scope to determine acceptance and trust impact
            let old_deliverable_count = old_promise.deliverables.len().max(1);
            let new_deliverable_count = new_promise.deliverables.len().max(1);
            let deliverable_change_ratio =
                (new_deliverable_count as f64 - old_deliverable_count as f64).abs()
                    / old_deliverable_count as f64;

            // Count prior renegotiations — repeated renegotiation erodes trust faster
            let prior_count = commitment.renegotiations.len();
            let fatigue_penalty = (prior_count as f64 * 0.03).min(0.2);

            // Reject if deliverable scope changed by more than 50%
            let accepted = deliverable_change_ratio <= 0.5;

            // Trust impact scales with change magnitude and renegotiation fatigue
            let trust_impact = if accepted {
                -(0.02 + deliverable_change_ratio * 0.08 + fatigue_penalty)
            } else {
                -(0.1 + fatigue_penalty)
            };

            commitment.renegotiations.push(Renegotiation {
                renegotiated_at: now,
                original: old_promise,
                new: new_promise.clone(),
                reason,
                accepted,
                trust_impact,
            });

            if accepted {
                commitment.promise = new_promise;
                commitment.status = CommitmentStatus::Active;
            } else {
                commitment.status = CommitmentStatus::AtRisk;
            }
            commitment.clone()
        };
        self.mark_dirty();
        Ok(out)
    }

    pub fn entangle_commitments(
        &mut self,
        a: CommitmentId,
        b: CommitmentId,
        entanglement_type: EntanglementType,
        strength: f64,
    ) -> Result<()> {
        if let Some(ca) = self.commitment_store.get_mut(&a) {
            ca.entanglements.push(CommitmentEntanglement {
                with: b,
                entanglement_type,
                strength,
            });
        }

        if let Some(cb) = self.commitment_store.get_mut(&b) {
            cb.entanglements.push(CommitmentEntanglement {
                with: a,
                entanglement_type,
                strength,
            });
        }

        self.mark_dirty();
        Ok(())
    }

    pub fn dream_goal(&mut self, id: GoalId) -> Result<Dream> {
        let goal = self
            .goal_store
            .get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone();

        let dream_id = DreamId(Uuid::new_v4());
        let scenario = self.generate_completion_scenario(&goal);
        let obstacles = self.predict_obstacles(&goal);
        let insights = self.extract_insights(&goal, &scenario, &obstacles);
        let discovered_goals = self.discover_sub_goals(&goal, &obstacles);

        let dream = Dream {
            id: dream_id,
            goal_id: id,
            dreamt_at: Timestamp::now(),
            scenario,
            obstacles,
            insights,
            discovered_goals: discovered_goals.clone(),
            confidence: self.calculate_dream_confidence(&goal),
            accuracy: None,
        };

        if let Some(goal_mut) = self.goal_store.get_mut(&id) {
            goal_mut.dreams.push(dream_id);
        }

        for seed in discovered_goals {
            if dream.confidence > 0.7 {
                let request = CreateGoalRequest {
                    title: seed.title,
                    description: seed.description,
                    parent: Some(id),
                    intention: seed.reason,
                    origin: Some(ProvenanceOrigin::Dream { dream: dream_id }),
                    ..Default::default()
                };
                let _ = self.create_goal(request);
            }
        }

        self.dream_store.insert(dream_id, dream.clone());
        self.mark_dirty();
        Ok(dream)
    }

    pub fn create_federation(
        &mut self,
        goal_id: GoalId,
        agent_id: String,
        coordinator: Option<String>,
    ) -> Result<Federation> {
        let goal = self
            .goal_store
            .get(&goal_id)
            .ok_or(Error::GoalNotFound(goal_id))?
            .clone();

        let now = Timestamp::now();
        let id = FederationId(Uuid::new_v4());
        let coordinator_id = coordinator.unwrap_or_else(|| agent_id.clone());

        let federation = Federation {
            id,
            goal_id,
            created_at: now,
            members: vec![FederationMember {
                agent_id: agent_id.clone(),
                joined_at: now,
                owned_goals: vec![goal_id],
                progress: goal.progress.percentage,
                status: MemberStatus::Active,
                last_active: now,
            }],
            coordinator: Some(coordinator_id),
            last_sync: now,
            sync_status: SyncStatus::Pending,
            collective_dreams: Vec::new(),
        };

        self.federation_store.insert(id, federation.clone());
        self.mark_dirty();
        Ok(federation)
    }

    pub fn join_federation(
        &mut self,
        federation_id: FederationId,
        agent_id: String,
    ) -> Result<Federation> {
        let now = Timestamp::now();
        let federation = self
            .federation_store
            .get_mut(&federation_id)
            .ok_or(Error::FederationNotFound(federation_id))?;

        if let Some(member) = federation
            .members
            .iter_mut()
            .find(|m| m.agent_id == agent_id)
        {
            member.status = MemberStatus::Active;
            member.last_active = now;
        } else {
            federation.members.push(FederationMember {
                agent_id,
                joined_at: now,
                owned_goals: vec![federation.goal_id],
                progress: 0.0,
                status: MemberStatus::Active,
                last_active: now,
            });
        }

        federation.sync_status = SyncStatus::Pending;
        federation.last_sync = now;
        let out = federation.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn sync_federation(&mut self, federation_id: FederationId) -> Result<Federation> {
        let now = Timestamp::now();
        let goal_progress = self
            .federation_store
            .get(&federation_id)
            .and_then(|f| self.goal_store.get(&f.goal_id))
            .map(|goal| goal.progress.percentage)
            .unwrap_or(0.0);

        let federation = self
            .federation_store
            .get_mut(&federation_id)
            .ok_or(Error::FederationNotFound(federation_id))?;

        for member in &mut federation.members {
            member.last_active = now;
            member.progress = member.progress.max(goal_progress);
            if matches!(
                member.status,
                MemberStatus::Blocked | MemberStatus::Inactive
            ) {
                member.status = MemberStatus::Active;
            }
        }

        federation.last_sync = now;
        federation.sync_status = SyncStatus::Synced;

        let out = federation.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn handoff_federation(
        &mut self,
        federation_id: FederationId,
        next_coordinator: String,
    ) -> Result<Federation> {
        let now = Timestamp::now();
        let federation = self
            .federation_store
            .get_mut(&federation_id)
            .ok_or(Error::FederationNotFound(federation_id))?;

        if let Some(member) = federation
            .members
            .iter_mut()
            .find(|m| m.agent_id == next_coordinator)
        {
            member.last_active = now;
            member.status = MemberStatus::Active;
        } else {
            federation.members.push(FederationMember {
                agent_id: next_coordinator.clone(),
                joined_at: now,
                owned_goals: vec![federation.goal_id],
                progress: 0.0,
                status: MemberStatus::Active,
                last_active: now,
            });
        }

        federation.coordinator = Some(next_coordinator);
        federation.sync_status = SyncStatus::Pending;
        federation.last_sync = now;
        let out = federation.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn detect_metamorphosis(&self, goal_id: GoalId) -> Result<MetamorphosisSignal> {
        let goal = self
            .goal_store
            .get(&goal_id)
            .ok_or(Error::GoalNotFound(goal_id))?;

        let (should_transform, reason, change) =
            if goal.status == GoalStatus::Blocked && !goal.blockers.is_empty() {
                (
                    true,
                    "Goal is blocked with active blockers".to_string(),
                    ScopeChange::Pivot {
                        new_direction: "Remove blockers via alternate path".to_string(),
                        reason: "Sustained blockage".to_string(),
                    },
                )
            } else if goal.physics.momentum > 0.75 && goal.progress.percentage > 0.4 {
                (
                    true,
                    "Momentum indicates expansion opportunity".to_string(),
                    ScopeChange::Expansion {
                        factor: 1.25,
                        reason: "Execution strength supports broader scope".to_string(),
                    },
                )
            } else if goal.feelings.neglect > 0.6 && goal.progress.percentage < 0.2 {
                (
                    true,
                    "Neglect drift suggests refinement".to_string(),
                    ScopeChange::Refinement {
                        clarification: "Narrow scope into immediate deliverable".to_string(),
                    },
                )
            } else {
                (
                    false,
                    "No metamorphosis signal detected".to_string(),
                    ScopeChange::Contraction {
                        factor: 0.95,
                        reason: "Hold current scope".to_string(),
                    },
                )
            };

        Ok(MetamorphosisSignal {
            goal_id,
            should_transform,
            reason,
            recommended_change: change,
        })
    }

    pub fn approve_metamorphosis(
        &mut self,
        goal_id: GoalId,
        stage_title: String,
        stage_description: String,
        change: ScopeChange,
    ) -> Result<Goal> {
        let now = Timestamp::now();
        let goal = self
            .goal_store
            .get_mut(&goal_id)
            .ok_or(Error::GoalNotFound(goal_id))?;

        if goal.metamorphosis.is_none() {
            goal.metamorphosis = Some(GoalMetamorphosis {
                stages: Vec::new(),
                current_stage: 0,
                invariant_soul: goal.soul.clone(),
            });
        }

        if let Some(meta) = &mut goal.metamorphosis {
            let stage_number = meta.stages.len() + 1;
            meta.stages.push(MetamorphicStage {
                stage_number,
                title: stage_title,
                description: stage_description,
                entered_at: now,
                scope_change: change,
            });
            meta.current_stage = meta.stages.len().saturating_sub(1);
        }

        let out = goal.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn metamorphosis_history(&self, goal_id: GoalId) -> Result<Vec<MetamorphicStage>> {
        let goal = self
            .goal_store
            .get(&goal_id)
            .ok_or(Error::GoalNotFound(goal_id))?;
        Ok(goal
            .metamorphosis
            .as_ref()
            .map(|m| m.stages.clone())
            .unwrap_or_default())
    }

    pub fn predict_metamorphosis(&self, goal_id: GoalId) -> Result<MetamorphosisPrediction> {
        let signal = self.detect_metamorphosis(goal_id)?;
        let goal = self
            .goal_store
            .get(&goal_id)
            .ok_or(Error::GoalNotFound(goal_id))?;
        let confidence =
            ((goal.physics.momentum + goal.feelings.urgency + goal.feelings.confidence) / 3.0)
                .clamp(0.1, 1.0);

        Ok(MetamorphosisPrediction {
            goal_id,
            confidence,
            next_change: signal.recommended_change,
            rationale: signal.reason,
        })
    }

    pub fn metamorphosis_stage(&self, goal_id: GoalId) -> Result<Option<MetamorphicStage>> {
        let goal = self
            .goal_store
            .get(&goal_id)
            .ok_or(Error::GoalNotFound(goal_id))?;
        let stage = goal
            .metamorphosis
            .as_ref()
            .and_then(|m| m.stages.get(m.current_stage).cloned());
        Ok(stage)
    }

    pub fn merge_from(&mut self, source: &PlanningEngine) -> MergeReport {
        let mut report = MergeReport::default();

        for (id, goal) in &source.goal_store {
            if self.goal_store.insert(*id, goal.clone()).is_none() {
                report.goals_merged += 1;
            }
        }
        for (id, decision) in &source.decision_store {
            if self.decision_store.insert(*id, decision.clone()).is_none() {
                report.decisions_merged += 1;
            }
        }
        for (id, commitment) in &source.commitment_store {
            if self
                .commitment_store
                .insert(*id, commitment.clone())
                .is_none()
            {
                report.commitments_merged += 1;
            }
        }
        for (id, dream) in &source.dream_store {
            if self.dream_store.insert(*id, dream.clone()).is_none() {
                report.dreams_merged += 1;
            }
        }
        for (id, federation) in &source.federation_store {
            if self
                .federation_store
                .insert(*id, federation.clone())
                .is_none()
            {
                report.federations_merged += 1;
            }
        }
        for (id, soul) in &source.soul_archive {
            if self.soul_archive.insert(*id, soul.clone()).is_none() {
                report.souls_merged += 1;
            }
        }

        self.rebuild_indexes();
        self.mark_dirty();
        report
    }

    // --- Missing write operations (SPEC-PART2) ---

    pub fn update_goal(&mut self, id: GoalId, updates: UpdateGoalRequest) -> Result<Goal> {
        let goal = self
            .goal_store
            .get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;

        if let Some(title) = updates.title {
            if title.is_empty() {
                return Err(Error::Validation("title cannot be empty".to_string()));
            }
            goal.title = title;
        }
        if let Some(description) = updates.description {
            goal.description = description;
        }
        if let Some(deadline) = updates.deadline {
            goal.deadline = deadline;
        }
        if let Some(priority) = updates.priority {
            let old_priority = goal.priority;
            goal.priority = priority;
            self.indexes
                .goal_priority_changed(id, old_priority, priority);
        }
        if let Some(tags) = updates.tags {
            goal.tags = tags;
        }
        if let Some(metadata) = updates.metadata {
            goal.metadata = metadata;
        }
        if let Some(intention) = updates.intention {
            goal.soul.intention = intention;
        }
        if let Some(significance) = updates.significance {
            goal.soul.significance = significance;
        }
        if let Some(emotional_weight) = updates.emotional_weight {
            goal.soul.emotional_weight = emotional_weight.clamp(0.0, 1.0);
        }

        let out = goal.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn update_regret(&mut self, id: DecisionId) -> Result<Decision> {
        let decision = self
            .decision_store
            .get(&id)
            .ok_or(Error::DecisionNotFound(id))?
            .clone();

        let regret = self.calculate_regret(&decision);
        let decision_mut = self
            .decision_store
            .get_mut(&id)
            .ok_or(Error::DecisionNotFound(id))?;
        decision_mut.regret_score = regret;
        decision_mut.regret_updated_at = Some(Timestamp::now());

        if regret > 0.7 && decision_mut.status == DecisionStatus::Crystallized {
            decision_mut.status = DecisionStatus::Regretted;
        }

        let out = decision_mut.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn record_insight(&mut self, dream_id: DreamId, insight: DreamInsight) -> Result<Dream> {
        let dream = self
            .dream_store
            .get_mut(&dream_id)
            .ok_or(Error::DreamNotFound(dream_id))?;
        dream.insights.push(insight);
        let out = dream.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn assess_accuracy(&mut self, dream_id: DreamId, accuracy: DreamAccuracy) -> Result<Dream> {
        let dream = self
            .dream_store
            .get_mut(&dream_id)
            .ok_or(Error::DreamNotFound(dream_id))?;
        dream.accuracy = Some(accuracy);
        let out = dream.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn update_momentum(&mut self, id: GoalId) -> Result<Goal> {
        let momentum = {
            let goal = self.goal_store.get(&id).ok_or(Error::GoalNotFound(id))?;
            self.calculate_momentum_from_goal(goal)
        };
        let goal = self
            .goal_store
            .get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        goal.physics.momentum = momentum;
        goal.physics.last_calculated = Timestamp::now();
        let out = goal.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn update_gravity(&mut self, id: GoalId) -> Result<Goal> {
        let goal = self
            .goal_store
            .get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;

        let now = Timestamp::now();
        let deadline_pull = goal
            .deadline
            .map(|d| {
                let days_until = ((d.0 - now.0) as f64 / (86_400.0 * 1e9)).max(0.01);
                (1.0 / days_until).clamp(0.0, 1.0)
            })
            .unwrap_or(0.0);

        let priority_pull = match goal.priority {
            Priority::Critical => 0.9,
            Priority::High => 0.7,
            Priority::Medium => 0.5,
            Priority::Low => 0.3,
            Priority::Someday => 0.1,
        };

        let dependent_pull = (goal.dependents.len() as f64 * 0.1).clamp(0.0, 0.5);
        goal.physics.gravity =
            (deadline_pull * 0.4 + priority_pull * 0.35 + dependent_pull * 0.25).clamp(0.0, 1.0);
        goal.physics.last_calculated = now;

        let out = goal.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn update_feelings(&mut self, id: GoalId) -> Result<Goal> {
        let now = Timestamp::now();
        let (momentum, confidence) = {
            let goal = self.goal_store.get(&id).ok_or(Error::GoalNotFound(id))?;
            (
                self.calculate_momentum_from_goal(goal),
                self.calculate_confidence_from_goal(goal),
            )
        };

        let goal = self
            .goal_store
            .get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;

        // Urgency: increases with deadline proximity and priority
        let deadline_urgency = goal
            .deadline
            .map(|d| {
                let days_until = ((d.0 - now.0) as f64 / (86_400.0 * 1e9)).max(0.01);
                (7.0 / days_until).clamp(0.0, 1.0)
            })
            .unwrap_or(0.2);
        let priority_urgency = match goal.priority {
            Priority::Critical => 0.9,
            Priority::High => 0.7,
            Priority::Medium => 0.4,
            Priority::Low => 0.2,
            Priority::Someday => 0.05,
        };
        goal.feelings.urgency = (deadline_urgency * 0.6 + priority_urgency * 0.4).clamp(0.0, 1.0);

        // Neglect: increases the longer since last progress
        let last_progress_days = goal
            .progress
            .history
            .last()
            .map(|p| ((now.0 - p.timestamp.0) as f64 / (86_400.0 * 1e9)).max(0.0))
            .unwrap_or(30.0);
        goal.feelings.neglect = (last_progress_days / 14.0).clamp(0.0, 1.0);

        // Confidence: from blocker analysis
        goal.feelings.confidence = confidence;

        // Alignment: how well progress tracks with intentions
        let criteria_met = goal
            .soul
            .success_criteria
            .iter()
            .filter(|c| c.achieved)
            .count() as f64;
        let criteria_total = goal.soul.success_criteria.len().max(1) as f64;
        goal.feelings.alignment =
            (criteria_met / criteria_total * 0.5 + goal.progress.percentage * 0.5).clamp(0.0, 1.0);

        // Vitality: combination of momentum and inverse neglect
        goal.feelings.vitality =
            (momentum * 0.6 + (1.0 - goal.feelings.neglect) * 0.4).clamp(0.0, 1.0);

        goal.feelings.last_calculated = now;
        let out = goal.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn update_commitment(
        &mut self,
        id: CommitmentId,
        updates: UpdateCommitmentRequest,
    ) -> Result<Commitment> {
        let commitment = self
            .commitment_store
            .get_mut(&id)
            .ok_or(Error::CommitmentNotFound(id))?;

        if let Some(promise) = updates.promise {
            commitment.promise = promise;
        }
        if let Some(due) = updates.due {
            commitment.due = due;
        }
        if let Some(goal) = updates.goal {
            commitment.goal = goal;
        }

        let out = commitment.clone();
        self.mark_dirty();
        Ok(out)
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate goal references
        for (id, goal) in &self.goal_store {
            if goal.title.is_empty() {
                errors.push(format!("Goal {:?} has empty title", id));
            }
            if let Some(parent) = goal.parent {
                if !self.goal_store.contains_key(&parent) {
                    errors.push(format!(
                        "Goal {:?} references missing parent {:?}",
                        id, parent
                    ));
                }
            }
            for dep in &goal.dependencies {
                if !self.goal_store.contains_key(dep) {
                    errors.push(format!("Goal {:?} depends on missing goal {:?}", id, dep));
                }
            }
            for child in &goal.children {
                if !self.goal_store.contains_key(child) {
                    errors.push(format!(
                        "Goal {:?} references missing child {:?}",
                        id, child
                    ));
                }
            }
            for decision_id in &goal.decisions {
                if !self.decision_store.contains_key(decision_id) {
                    errors.push(format!(
                        "Goal {:?} references missing decision {:?}",
                        id, decision_id
                    ));
                }
            }
            for commitment_id in &goal.commitments {
                if !self.commitment_store.contains_key(commitment_id) {
                    errors.push(format!(
                        "Goal {:?} references missing commitment {:?}",
                        id, commitment_id
                    ));
                }
            }
            for dream_id in &goal.dreams {
                if !self.dream_store.contains_key(dream_id) {
                    errors.push(format!(
                        "Goal {:?} references missing dream {:?}",
                        id, dream_id
                    ));
                }
            }
            if !(0.0..=1.0).contains(&goal.progress.percentage) {
                errors.push(format!(
                    "Goal {:?} has invalid progress: {}",
                    id, goal.progress.percentage
                ));
            }
        }

        // Validate decision references
        for (id, decision) in &self.decision_store {
            for goal_id in &decision.affected_goals {
                if !self.goal_store.contains_key(goal_id) {
                    errors.push(format!(
                        "Decision {:?} references missing goal {:?}",
                        id, goal_id
                    ));
                }
            }
            if let Some(caused_by) = decision.caused_by {
                if !self.decision_store.contains_key(&caused_by) {
                    errors.push(format!(
                        "Decision {:?} references missing parent decision {:?}",
                        id, caused_by
                    ));
                }
            }
        }

        // Validate commitment references
        for (id, commitment) in &self.commitment_store {
            if let Some(goal_id) = commitment.goal {
                if !self.goal_store.contains_key(&goal_id) {
                    errors.push(format!(
                        "Commitment {:?} references missing goal {:?}",
                        id, goal_id
                    ));
                }
            }
            for ent in &commitment.entanglements {
                if !self.commitment_store.contains_key(&ent.with) {
                    errors.push(format!(
                        "Commitment {:?} entangled with missing commitment {:?}",
                        id, ent.with
                    ));
                }
            }
        }

        // Validate dream references
        for (id, dream) in &self.dream_store {
            if !self.goal_store.contains_key(&dream.goal_id) {
                errors.push(format!(
                    "Dream {:?} references missing goal {:?}",
                    id, dream.goal_id
                ));
            }
        }

        // Validate federation references
        for (id, federation) in &self.federation_store {
            if !self.goal_store.contains_key(&federation.goal_id) {
                errors.push(format!(
                    "Federation {:?} references missing goal {:?}",
                    id, federation.goal_id
                ));
            }
        }

        errors
    }
}
