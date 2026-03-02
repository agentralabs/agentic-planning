# AGENTIC PLANNING SPECIFICATION — PART 2

> **Specs Covered:** SPEC-05 through SPEC-08
> **Sister:** #8 of 25
> **Continues from:** Part 1

---

# SPEC-05: WRITE ENGINE

## 5.1 Overview

The Write Engine handles all mutations to the planning state:

```
WRITE OPERATIONS:
═════════════════

GOALS:
  create_goal          → Birth a new goal
  update_goal          → Modify goal properties
  progress_goal        → Record progress
  block_goal           → Add a blocker
  unblock_goal         → Remove a blocker
  complete_goal        → Mark as completed
  abandon_goal         → Mark as abandoned
  pause_goal           → Temporarily suspend
  resume_goal          → Resume paused goal
  reincarnate_goal     → Rebirth a dead goal
  decompose_goal       → Create sub-goals
  link_goals           → Create relationship

DECISIONS:
  create_decision      → Start a decision
  add_option           → Add a path option
  crystallize          → Make the choice
  record_consequence   → Log observed outcome
  update_regret        → Update regret score
  recrystallize        → Change decision (expensive)

COMMITMENTS:
  create_commitment    → Make a promise
  update_commitment    → Modify terms
  fulfill_commitment   → Mark as kept
  break_commitment     → Mark as broken
  renegotiate          → Change terms
  entangle             → Link commitments

DREAMS:
  dream_goal           → Simulate completion
  record_insight       → Add dream insight
  assess_accuracy      → Check dream predictions

PHYSICS:
  update_momentum      → Recalculate momentum
  update_gravity       → Recalculate gravity
  update_feelings      → Recalculate feelings
```

## 5.2 Goal Operations

```rust
impl PlanningEngine {
    // ========================================================================
    // GOAL CREATION
    // ========================================================================
    
    /// Birth a new goal
    pub fn create_goal(&mut self, request: CreateGoalRequest) -> Result<Goal> {
        let id = GoalId(Uuid::new_v4());
        let now = Timestamp::now();
        
        // Create soul
        let soul = GoalSoul {
            intention: request.intention.clone(),
            significance: request.significance.unwrap_or_default(),
            success_criteria: request.success_criteria.unwrap_or_default(),
            emotional_weight: request.emotional_weight.unwrap_or(0.5),
            values: request.values.unwrap_or_default(),
        };
        
        // Initial feelings (calm, no neglect, moderate confidence)
        let feelings = GoalFeelings {
            urgency: self.calculate_initial_urgency(&request),
            neglect: 0.0,
            confidence: 0.5,
            alignment: 1.0,
            vitality: 1.0,
            last_calculated: now,
        };
        
        // Initial physics
        let physics = GoalPhysics {
            momentum: 0.0,
            gravity: self.calculate_initial_gravity(&request),
            inertia: self.calculate_initial_inertia(&request),
            energy: 1.0,
            last_calculated: now,
        };
        
        // Create goal
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
        
        // Add to graph
        self.goal_store.insert(id, goal.clone());
        
        // Update parent if exists
        if let Some(parent_id) = request.parent {
            if let Some(parent) = self.goal_store.get_mut(&parent_id) {
                parent.children.push(id);
            }
        }
        
        // Update dependents
        for dep_id in &goal.dependencies {
            if let Some(dep) = self.goal_store.get_mut(dep_id) {
                dep.dependents.push(id);
            }
        }
        
        // Update indexes
        self.indexes.add_goal(&goal);
        
        // Persist
        self.mark_dirty();
        
        Ok(goal)
    }
    
    /// Activate a draft goal
    pub fn activate_goal(&mut self, id: GoalId) -> Result<Goal> {
        let goal = self.goal_store.get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        
        if goal.status != GoalStatus::Draft {
            return Err(Error::InvalidTransition {
                from: goal.status,
                to: GoalStatus::Active,
            });
        }
        
        goal.status = GoalStatus::Active;
        goal.activated_at = Some(Timestamp::now());
        goal.feelings.vitality = 1.0;
        
        self.indexes.goal_activated(id);
        self.mark_dirty();
        
        Ok(goal.clone())
    }
    
    /// Record progress on a goal
    pub fn progress_goal(
        &mut self,
        id: GoalId,
        percentage: f64,
        note: Option<String>,
    ) -> Result<Goal> {
        let now = Timestamp::now();
        let goal = self.goal_store.get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        
        // Validate percentage
        let percentage = percentage.clamp(0.0, 1.0);
        
        // Record progress point
        goal.progress.history.push(ProgressPoint {
            timestamp: now,
            percentage,
            note,
        });
        goal.progress.percentage = percentage;
        
        // Update velocity
        goal.progress.velocity = self.calculate_velocity(&goal.progress.history);
        
        // Update ETA
        if goal.progress.velocity > 0.0 {
            let remaining = 1.0 - percentage;
            let days_remaining = remaining / goal.progress.velocity;
            let eta_nanos = now.0 + (days_remaining * 86400.0 * 1e9) as i64;
            goal.progress.eta = Some(Timestamp(eta_nanos));
        }
        
        // Update physics
        goal.physics.momentum = self.calculate_momentum(goal);
        goal.feelings.neglect = 0.0;  // Just touched
        goal.feelings.confidence = self.calculate_confidence(goal);
        goal.feelings.last_calculated = now;
        
        // Check for completion
        if percentage >= 1.0 {
            self.check_success_criteria(goal);
        }
        
        self.mark_dirty();
        Ok(goal.clone())
    }
    
    /// Complete a goal
    pub fn complete_goal(&mut self, id: GoalId, note: Option<String>) -> Result<Goal> {
        let now = Timestamp::now();
        let goal = self.goal_store.get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        
        // Must be active
        if !matches!(goal.status, GoalStatus::Active | GoalStatus::Blocked) {
            return Err(Error::CannotComplete(goal.status));
        }
        
        goal.status = GoalStatus::Completed;
        goal.completed_at = Some(now);
        goal.progress.percentage = 1.0;
        goal.progress.history.push(ProgressPoint {
            timestamp: now,
            percentage: 1.0,
            note,
        });
        
        // Release energy to related goals
        self.release_completion_energy(id);
        
        // Unblock dependents
        for dependent_id in goal.dependents.clone() {
            self.check_unblock(&dependent_id);
        }
        
        self.indexes.goal_completed(id);
        self.mark_dirty();
        
        Ok(goal.clone())
    }
    
    /// Abandon a goal
    pub fn abandon_goal(&mut self, id: GoalId, reason: String) -> Result<Goal> {
        let now = Timestamp::now();
        let goal = self.goal_store.get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        
        // Preserve soul for potential reincarnation
        let karma = GoalKarma {
            failures: vec![reason.clone()],
            near_successes: Vec::new(),
            requirements_for_success: Vec::new(),
            invested_energy: goal.progress.percentage,
        };
        
        // Archive soul
        self.soul_archive.insert(id, GoalSoulArchive {
            original_id: id,
            soul: goal.soul.clone(),
            death_record: GoalDeath {
                cause: reason,
                timestamp: now,
            },
            karma,
            reincarnation_potential: self.calculate_reincarnation_potential(goal),
            trigger_conditions: Vec::new(),
        });
        
        goal.status = GoalStatus::Abandoned;
        goal.completed_at = Some(now);
        goal.physics.momentum = 0.0;
        goal.physics.energy = 0.0;
        
        self.indexes.goal_abandoned(id);
        self.mark_dirty();
        
        Ok(goal.clone())
    }
    
    /// Reincarnate a dead goal
    pub fn reincarnate_goal(
        &mut self,
        original_id: GoalId,
        updates: ReincarnationUpdates,
    ) -> Result<Goal> {
        // Get archived soul
        let archive = self.soul_archive.get(&original_id)
            .ok_or(Error::SoulNotFound(original_id))?
            .clone();
        
        // Create new goal with preserved soul
        let mut request = CreateGoalRequest {
            title: updates.title.unwrap_or_else(|| {
                format!("{} (Reborn)", archive.soul.intention)
            }),
            description: updates.description.unwrap_or_default(),
            intention: archive.soul.intention.clone(),
            significance: Some(archive.soul.significance.clone()),
            success_criteria: Some(archive.soul.success_criteria.clone()),
            emotional_weight: Some(archive.soul.emotional_weight),
            values: Some(archive.soul.values.clone()),
            origin: Some(ProvenanceOrigin::Reincarnation { previous: original_id }),
            ..Default::default()
        };
        
        let mut goal = self.create_goal(request)?;
        
        // Add previous life record
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
    
    /// Decompose a goal into sub-goals
    pub fn decompose_goal(
        &mut self,
        id: GoalId,
        sub_goals: Vec<CreateGoalRequest>,
    ) -> Result<Vec<Goal>> {
        let parent = self.goal_store.get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone();
        
        let mut created = Vec::new();
        
        for mut request in sub_goals {
            request.parent = Some(id);
            request.origin = Some(ProvenanceOrigin::Decomposition { parent: id });
            
            let child = self.create_goal(request)?;
            created.push(child);
        }
        
        Ok(created)
    }
    
    /// Add a blocker to a goal
    pub fn block_goal(&mut self, id: GoalId, blocker: Blocker) -> Result<Goal> {
        let goal = self.goal_store.get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        
        goal.blockers.push(blocker);
        
        if goal.status == GoalStatus::Active {
            goal.status = GoalStatus::Blocked;
            self.indexes.goal_blocked(id);
        }
        
        goal.feelings.confidence = self.calculate_confidence(goal);
        
        self.mark_dirty();
        Ok(goal.clone())
    }
    
    /// Remove a blocker from a goal
    pub fn unblock_goal(
        &mut self,
        id: GoalId,
        blocker_id: Uuid,
        resolution: String,
    ) -> Result<Goal> {
        let now = Timestamp::now();
        let goal = self.goal_store.get_mut(&id)
            .ok_or(Error::GoalNotFound(id))?;
        
        // Find and resolve blocker
        for blocker in &mut goal.blockers {
            if blocker.id == blocker_id {
                blocker.resolved_at = Some(now);
                blocker.resolution = Some(resolution);
            }
        }
        
        // Check if all blockers resolved
        let all_resolved = goal.blockers.iter()
            .all(|b| b.resolved_at.is_some());
        
        if all_resolved && goal.status == GoalStatus::Blocked {
            goal.status = GoalStatus::Active;
            self.indexes.goal_unblocked(id);
        }
        
        goal.feelings.confidence = self.calculate_confidence(goal);
        
        self.mark_dirty();
        Ok(goal.clone())
    }
    
    /// Create a relationship between goals
    pub fn link_goals(
        &mut self,
        relationship: GoalRelationship,
    ) -> Result<()> {
        match &relationship {
            GoalRelationship::Alliance { goals: (a, b), .. } |
            GoalRelationship::Rivalry { goals: (a, b), .. } |
            GoalRelationship::Romance { goals: (a, b), .. } |
            GoalRelationship::Nemesis { goals: (a, b), .. } => {
                // Add to both goals
                if let Some(goal_a) = self.goal_store.get_mut(a) {
                    goal_a.relationships.push(relationship.clone());
                }
                if let Some(goal_b) = self.goal_store.get_mut(b) {
                    goal_b.relationships.push(relationship);
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
            _ => {}
        }
        
        self.mark_dirty();
        Ok(())
    }
}
```

## 5.3 Decision Operations

```rust
impl PlanningEngine {
    // ========================================================================
    // DECISION OPERATIONS
    // ========================================================================
    
    /// Create a new decision
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
        
        // Link to goals
        for goal_id in &decision.affected_goals {
            if let Some(goal) = self.goal_store.get_mut(goal_id) {
                goal.decisions.push(id);
            }
        }
        
        // Link to parent decision
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
    
    /// Add an option to a decision
    pub fn add_option(&mut self, id: DecisionId, path: DecisionPath) -> Result<Decision> {
        let decision = self.decision_store.get_mut(&id)
            .ok_or(Error::DecisionNotFound(id))?;
        
        if decision.status == DecisionStatus::Crystallized {
            return Err(Error::AlreadyCrystallized);
        }
        
        decision.status = DecisionStatus::Deliberating;
        
        // Add as shadow (will become chosen or stay shadow)
        decision.shadows.push(CrystalShadow {
            path,
            rejection_reason: String::new(),
            counterfactual: None,
            resurrection_cost: 0.0,
        });
        
        self.mark_dirty();
        Ok(decision.clone())
    }
    
    /// Crystallize a decision
    pub fn crystallize(
        &mut self,
        id: DecisionId,
        chosen_path_id: PathId,
        reasoning: DecisionReasoning,
    ) -> Result<Decision> {
        let now = Timestamp::now();
        let decision = self.decision_store.get_mut(&id)
            .ok_or(Error::DecisionNotFound(id))?;
        
        if decision.status == DecisionStatus::Crystallized {
            return Err(Error::AlreadyCrystallized);
        }
        
        // Find chosen path and remove from shadows
        let chosen_idx = decision.shadows.iter()
            .position(|s| s.path.id == chosen_path_id)
            .ok_or(Error::PathNotFound(chosen_path_id))?;
        
        let chosen_shadow = decision.shadows.remove(chosen_idx);
        decision.chosen = Some(chosen_shadow.path);
        
        // Mark remaining as shadows with rejection reasons
        for shadow in &mut decision.shadows {
            if shadow.rejection_reason.is_empty() {
                shadow.rejection_reason = "Not chosen".to_string();
            }
            // Calculate resurrection cost
            shadow.resurrection_cost = self.calculate_resurrection_cost(decision, shadow);
        }
        
        decision.status = DecisionStatus::Crystallized;
        decision.crystallized_at = Some(now);
        decision.reasoning = reasoning;
        decision.reversibility = self.calculate_reversibility(decision);
        
        self.indexes.decision_crystallized(id);
        self.mark_dirty();
        
        Ok(decision.clone())
    }
    
    /// Record a consequence of a decision
    pub fn record_consequence(
        &mut self,
        id: DecisionId,
        consequence: Consequence,
    ) -> Result<Decision> {
        let decision = self.decision_store.get_mut(&id)
            .ok_or(Error::DecisionNotFound(id))?;
        
        decision.consequences.push(consequence);
        
        // Update regret score based on consequences
        decision.regret_score = self.calculate_regret(decision);
        decision.regret_updated_at = Some(Timestamp::now());
        
        if decision.regret_score > 0.7 {
            decision.status = DecisionStatus::Regretted;
        }
        
        self.mark_dirty();
        Ok(decision.clone())
    }
    
    /// Recrystallize (change) a decision
    pub fn recrystallize(
        &mut self,
        id: DecisionId,
        new_path_id: PathId,
        reason: String,
    ) -> Result<Decision> {
        let now = Timestamp::now();
        let decision = self.decision_store.get_mut(&id)
            .ok_or(Error::DecisionNotFound(id))?;
        
        if decision.status != DecisionStatus::Crystallized &&
           decision.status != DecisionStatus::Regretted {
            return Err(Error::CannotRecrystallize(decision.status));
        }
        
        // Move current chosen to shadows
        if let Some(old_chosen) = decision.chosen.take() {
            decision.shadows.push(CrystalShadow {
                path: old_chosen,
                rejection_reason: format!("Recrystallized: {}", reason),
                counterfactual: None,
                resurrection_cost: 0.0,
            });
        }
        
        // Find new path in shadows
        let new_idx = decision.shadows.iter()
            .position(|s| s.path.id == new_path_id)
            .ok_or(Error::PathNotFound(new_path_id))?;
        
        let new_chosen = decision.shadows.remove(new_idx);
        decision.chosen = Some(new_chosen.path);
        decision.status = DecisionStatus::Recrystallized;
        decision.crystallized_at = Some(now);
        decision.regret_score = 0.0;
        
        // Cascade to dependent decisions
        self.cascade_recrystallization(id)?;
        
        self.mark_dirty();
        Ok(decision.clone())
    }
}
```

## 5.4 Commitment Operations

```rust
impl PlanningEngine {
    // ========================================================================
    // COMMITMENT OPERATIONS
    // ========================================================================
    
    /// Create a commitment
    pub fn create_commitment(&mut self, request: CreateCommitmentRequest) -> Result<Commitment> {
        let id = CommitmentId(Uuid::new_v4());
        let now = Timestamp::now();
        
        let commitment = Commitment {
            id,
            promise: request.promise,
            made_to: request.stakeholder,
            made_at: now,
            due: request.due,
            status: CommitmentStatus::Active,
            weight: self.calculate_commitment_weight(&request),
            inertia: self.calculate_commitment_inertia(&request),
            breaking_cost: self.calculate_breaking_cost(&request),
            goal: request.goal,
            entanglements: Vec::new(),
            fulfillment: None,
            renegotiations: Vec::new(),
        };
        
        // Link to goal
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
    
    /// Fulfill a commitment
    pub fn fulfill_commitment(
        &mut self,
        id: CommitmentId,
        how_delivered: String,
    ) -> Result<Commitment> {
        let now = Timestamp::now();
        let commitment = self.commitment_store.get_mut(&id)
            .ok_or(Error::CommitmentNotFound(id))?;
        
        if commitment.status != CommitmentStatus::Active {
            return Err(Error::CannotFulfill(commitment.status));
        }
        
        // Calculate energy released
        let energy_released = commitment.weight * 
            (1.0 + self.calculate_chain_bonus(id));
        
        commitment.status = CommitmentStatus::Fulfilled;
        commitment.fulfillment = Some(CommitmentFulfillment {
            fulfilled_at: now,
            how_delivered,
            energy_released,
            trust_gained: commitment.weight * 0.5,
        });
        
        // Release energy to entangled commitments
        for entanglement in &commitment.entanglements {
            if entanglement.entanglement_type == EntanglementType::Sequential {
                self.boost_commitment(entanglement.with, energy_released * 0.3)?;
            }
        }
        
        self.indexes.commitment_fulfilled(id);
        self.mark_dirty();
        
        Ok(commitment.clone())
    }
    
    /// Break a commitment
    pub fn break_commitment(
        &mut self,
        id: CommitmentId,
        reason: String,
    ) -> Result<Commitment> {
        let commitment = self.commitment_store.get_mut(&id)
            .ok_or(Error::CommitmentNotFound(id))?;
        
        commitment.status = CommitmentStatus::Broken;
        
        // Apply breaking cost
        // (In practice, this would affect trust scores, etc.)
        
        // Cascade to entangled commitments
        for entanglement in commitment.entanglements.clone() {
            if entanglement.entanglement_type == EntanglementType::Parallel {
                self.destabilize_commitment(entanglement.with)?;
            }
        }
        
        self.indexes.commitment_broken(id);
        self.mark_dirty();
        
        Ok(commitment.clone())
    }
    
    /// Renegotiate a commitment
    pub fn renegotiate_commitment(
        &mut self,
        id: CommitmentId,
        new_promise: Promise,
        reason: String,
    ) -> Result<Commitment> {
        let now = Timestamp::now();
        let commitment = self.commitment_store.get_mut(&id)
            .ok_or(Error::CommitmentNotFound(id))?;
        
        let old_promise = commitment.promise.clone();
        
        commitment.renegotiations.push(Renegotiation {
            renegotiated_at: now,
            original: old_promise,
            new: new_promise.clone(),
            reason,
            accepted: true,  // Would be set after stakeholder response
            trust_impact: -0.05,  // Small trust cost for renegotiation
        });
        
        commitment.promise = new_promise;
        commitment.status = CommitmentStatus::Active;
        
        self.mark_dirty();
        Ok(commitment.clone())
    }
    
    /// Entangle two commitments
    pub fn entangle_commitments(
        &mut self,
        a: CommitmentId,
        b: CommitmentId,
        entanglement_type: EntanglementType,
        strength: f64,
    ) -> Result<()> {
        // Add entanglement to both commitments
        if let Some(commit_a) = self.commitment_store.get_mut(&a) {
            commit_a.entanglements.push(CommitmentEntanglement {
                with: b,
                entanglement_type,
                strength,
            });
        }
        
        if let Some(commit_b) = self.commitment_store.get_mut(&b) {
            commit_b.entanglements.push(CommitmentEntanglement {
                with: a,
                entanglement_type,
                strength,
            });
        }
        
        self.mark_dirty();
        Ok(())
    }
}
```

## 5.5 Dream Operations

```rust
impl PlanningEngine {
    // ========================================================================
    // DREAM OPERATIONS
    // ========================================================================
    
    /// Trigger a goal to dream its completion
    pub fn dream_goal(&mut self, id: GoalId) -> Result<Dream> {
        let goal = self.goal_store.get(&id)
            .ok_or(Error::GoalNotFound(id))?
            .clone();
        
        let dream_id = DreamId(Uuid::new_v4());
        let now = Timestamp::now();
        
        // Generate dream based on goal state
        let scenario = self.generate_completion_scenario(&goal);
        let obstacles = self.predict_obstacles(&goal);
        let insights = self.extract_insights(&goal, &scenario, &obstacles);
        let discovered_goals = self.discover_sub_goals(&goal, &obstacles);
        
        let dream = Dream {
            id: dream_id,
            goal_id: id,
            dreamt_at: now,
            scenario,
            obstacles,
            insights,
            discovered_goals: discovered_goals.clone(),
            confidence: self.calculate_dream_confidence(&goal),
            accuracy: None,
        };
        
        // Link dream to goal
        if let Some(goal) = self.goal_store.get_mut(&id) {
            goal.dreams.push(dream_id);
        }
        
        // Auto-create discovered sub-goals if high confidence
        for seed in discovered_goals {
            if dream.confidence > 0.7 {
                let request = CreateGoalRequest {
                    title: seed.title,
                    description: seed.description,
                    parent: Some(id),
                    intention: seed.reason.clone(),
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
    
    /// Generate a completion scenario
    fn generate_completion_scenario(&self, goal: &Goal) -> CompletionScenario {
        // In practice, this would use LLM or heuristics
        CompletionScenario {
            vision: format!("{} has been achieved. All success criteria met.", goal.title),
            feeling: "Satisfaction and accomplishment".to_string(),
            world_changes: goal.soul.success_criteria.iter()
                .map(|c| format!("✓ {}", c.description))
                .collect(),
            stakeholder_reactions: HashMap::new(),
        }
    }
    
    /// Predict obstacles from dream
    fn predict_obstacles(&self, goal: &Goal) -> Vec<DreamObstacle> {
        let mut obstacles = Vec::new();
        
        // Check dependencies
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
        
        // Check resource conflicts (rival goals)
        for rel in &goal.relationships {
            if let GoalRelationship::Rivalry { goals: (_, rival), contested, .. } = rel {
                obstacles.push(DreamObstacle {
                    description: format!("Resource conflict over {:?}", contested),
                    severity: 0.5,
                    timing: "During execution".to_string(),
                    mitigation: Some("Resolve resource allocation".to_string()),
                });
            }
        }
        
        obstacles
    }
}
```

---

# SPEC-06: QUERY ENGINE

## 6.1 Overview

The Query Engine provides read access to planning state:

```
QUERY OPERATIONS:
═════════════════

GOALS:
  get_goal             → Single goal by ID
  list_goals           → All goals with filters
  get_goal_tree        → Goal hierarchy
  get_active_goals     → Currently active
  get_blocked_goals    → Currently blocked
  get_urgent_goals     → Due soon
  search_goals         → Full-text search

DECISIONS:
  get_decision         → Single decision by ID
  list_decisions       → All decisions with filters
  get_decision_chain   → Causality chain
  get_shadows          → Unchosen paths
  search_decisions     → Full-text search

COMMITMENTS:
  get_commitment       → Single commitment by ID
  list_commitments     → All commitments with filters
  get_due_soon         → Due within N days
  get_at_risk          → At-risk commitments

ANALYTICS:
  get_intention_singularity → Unified intention field
  get_progress_forecast     → Predicted completion
  get_blocker_prophecy      → Predicted blockers
  get_momentum_report       → Momentum analysis
  get_gravity_field         → Goal gravity visualization
```

## 6.2 Goal Queries

```rust
impl PlanningEngine {
    // ========================================================================
    // GOAL QUERIES
    // ========================================================================
    
    /// Get a single goal by ID
    pub fn get_goal(&self, id: GoalId) -> Option<&Goal> {
        self.goal_store.get(&id)
    }
    
    /// List goals with filters
    pub fn list_goals(&self, filter: GoalFilter) -> Vec<&Goal> {
        self.goal_store.values()
            .filter(|g| self.matches_goal_filter(g, &filter))
            .collect()
    }
    
    /// Get active goals
    pub fn get_active_goals(&self) -> Vec<&Goal> {
        self.indexes.active_goals.iter()
            .filter_map(|id| self.goal_store.get(id))
            .collect()
    }
    
    /// Get blocked goals
    pub fn get_blocked_goals(&self) -> Vec<&Goal> {
        self.indexes.blocked_goals.iter()
            .filter_map(|id| self.goal_store.get(id))
            .collect()
    }
    
    /// Get goals due soon
    pub fn get_urgent_goals(&self, within_days: f64) -> Vec<&Goal> {
        let now = Timestamp::now();
        let cutoff = Timestamp(now.0 + (within_days * 86400.0 * 1e9) as i64);
        
        self.indexes.goals_by_deadline.iter()
            .filter(|(deadline, _)| *deadline <= cutoff)
            .filter_map(|(_, id)| self.goal_store.get(id))
            .filter(|g| g.status == GoalStatus::Active || g.status == GoalStatus::Blocked)
            .collect()
    }
    
    /// Get goal tree (hierarchy)
    pub fn get_goal_tree(&self, root_id: GoalId) -> Option<GoalTree> {
        let root = self.goal_store.get(&root_id)?;
        
        let mut tree = GoalTree {
            root: root_id,
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        
        self.build_tree_recursive(root_id, &mut tree);
        
        Some(tree)
    }
    
    fn build_tree_recursive(&self, id: GoalId, tree: &mut GoalTree) {
        if let Some(goal) = self.goal_store.get(&id) {
            tree.nodes.insert(id, GoalTreeNode {
                goal: goal.clone(),
                depth: self.calculate_depth(id),
            });
            
            for child_id in &goal.children {
                tree.edges.push((id, *child_id));
                self.build_tree_recursive(*child_id, tree);
            }
        }
    }
    
    /// Search goals by text
    pub fn search_goals(&self, query: &str) -> Vec<&Goal> {
        let query_lower = query.to_lowercase();
        
        self.goal_store.values()
            .filter(|g| {
                g.title.to_lowercase().contains(&query_lower) ||
                g.description.to_lowercase().contains(&query_lower) ||
                g.soul.intention.to_lowercase().contains(&query_lower) ||
                g.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
    
    /// Get intention singularity (unified field)
    pub fn get_intention_singularity(&self) -> IntentionSingularity {
        let active_goals: Vec<_> = self.get_active_goals()
            .into_iter()
            .cloned()
            .collect();
        
        // Calculate center of gravity
        let center = self.calculate_intention_center(&active_goals);
        
        // Position each goal in intention space
        let positions = active_goals.iter()
            .map(|g| {
                let position = IntentionPosition {
                    goal_id: g.id,
                    centrality: self.calculate_centrality(g, &center),
                    alignment_angle: self.calculate_alignment(g, &center),
                    gravitational_pull: g.physics.gravity,
                    drift_risk: g.feelings.neglect,
                };
                (g.id, position)
            })
            .collect();
        
        // Find themes
        let themes = self.extract_themes(&active_goals);
        
        // Find tensions
        let tensions = self.find_tensions(&active_goals);
        
        // Calculate golden path
        let golden_path = self.calculate_optimal_path(&active_goals);
        
        IntentionSingularity {
            unified_vision: self.synthesize_vision(&active_goals),
            goal_positions: positions,
            themes,
            tension_lines: tensions,
            golden_path,
            center,
        }
    }
}

/// Goal filter for queries
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
```

## 6.3 Decision Queries

```rust
impl PlanningEngine {
    // ========================================================================
    // DECISION QUERIES
    // ========================================================================
    
    /// Get a single decision by ID
    pub fn get_decision(&self, id: DecisionId) -> Option<&Decision> {
        self.decision_store.get(&id)
    }
    
    /// Get decision chain (causality)
    pub fn get_decision_chain(&self, id: DecisionId) -> Option<DecisionChain> {
        let decision = self.decision_store.get(&id)?;
        
        // Trace backwards to root
        let mut root = id;
        while let Some(parent_id) = self.decision_store.get(&root)
            .and_then(|d| d.caused_by) {
            root = parent_id;
        }
        
        // Build chain forward from root
        let mut chain = DecisionChain {
            root,
            descendants: Vec::new(),
            causality: Vec::new(),
            cascade_analysis: CascadeAnalysis::default(),
        };
        
        self.build_chain_recursive(root, &mut chain);
        
        Some(chain)
    }
    
    fn build_chain_recursive(&self, id: DecisionId, chain: &mut DecisionChain) {
        if let Some(decision) = self.decision_store.get(&id) {
            for child_id in &decision.causes {
                chain.descendants.push(*child_id);
                chain.causality.push(CausalLink {
                    from: id,
                    to: *child_id,
                    causality_type: CausalityType::Enables,
                    strength: 1.0,
                });
                self.build_chain_recursive(*child_id, chain);
            }
        }
    }
    
    /// Get crystal shadows for a decision
    pub fn get_shadows(&self, id: DecisionId) -> Vec<&CrystalShadow> {
        self.decision_store.get(&id)
            .map(|d| d.shadows.iter().collect())
            .unwrap_or_default()
    }
    
    /// Project a counterfactual
    pub fn project_counterfactual(
        &self,
        decision_id: DecisionId,
        path_id: PathId,
    ) -> Option<CounterfactualProjection> {
        let decision = self.decision_store.get(&decision_id)?;
        
        // Find the shadow
        let shadow = decision.shadows.iter()
            .find(|s| s.path.id == path_id)?;
        
        // Generate projection
        Some(CounterfactualProjection {
            projected_at: Timestamp::now(),
            timeline: self.generate_projected_timeline(&decision, &shadow.path),
            final_state: self.project_final_state(&decision, &shadow.path),
            confidence: self.calculate_projection_confidence(&decision),
        })
    }
    
    /// Perform decision archaeology
    pub fn decision_archaeology(&self, artifact: &str) -> DecisionArchaeology {
        // Find all decisions related to the artifact
        let relevant_decisions: Vec<_> = self.decision_store.values()
            .filter(|d| {
                d.question.question.contains(artifact) ||
                d.question.context.contains(artifact)
            })
            .collect();
        
        // Sort by time (oldest first = deepest stratum)
        let mut sorted = relevant_decisions.clone();
        sorted.sort_by_key(|d| d.crystallized_at.unwrap_or(Timestamp(0)));
        
        // Build strata
        let strata: Vec<_> = sorted.iter()
            .enumerate()
            .map(|(i, d)| ArchaeologicalStratum {
                depth: sorted.len() - i,
                decision: d.id,
                age: self.calculate_age(d),
                impact_on_artifact: format!("{:?}", d.chosen),
                context_at_time: d.question.context.clone(),
                was_reasonable: true,  // Would analyze
                modern_assessment: "".to_string(),
            })
            .collect();
        
        DecisionArchaeology {
            artifact: artifact.to_string(),
            strata,
            cumulative_impact: self.calculate_cumulative_impact(&sorted),
            insights: Vec::new(),
        }
    }
}
```

## 6.4 Commitment Queries

```rust
impl PlanningEngine {
    // ========================================================================
    // COMMITMENT QUERIES
    // ========================================================================
    
    /// Get commitments due soon
    pub fn get_due_soon(&self, within_days: f64) -> Vec<&Commitment> {
        let now = Timestamp::now();
        let cutoff = Timestamp(now.0 + (within_days * 86400.0 * 1e9) as i64);
        
        self.indexes.commitments_by_due.iter()
            .filter(|(due, _)| *due <= cutoff)
            .filter_map(|(_, id)| self.commitment_store.get(id))
            .filter(|c| c.status == CommitmentStatus::Active)
            .collect()
    }
    
    /// Get commitment inventory with weights
    pub fn get_commitment_inventory(&self) -> CommitmentInventory {
        let commitments: Vec<_> = self.commitment_store.values().collect();
        
        let total_weight: f64 = commitments.iter()
            .filter(|c| c.status == CommitmentStatus::Active)
            .map(|c| c.weight)
            .sum();
        
        CommitmentInventory {
            total_count: commitments.len(),
            active_count: commitments.iter()
                .filter(|c| c.status == CommitmentStatus::Active)
                .count(),
            total_weight,
            sustainable_weight: 2.0,  // Configurable
            is_overloaded: total_weight > 2.0,
            by_stakeholder: self.group_by_stakeholder(&commitments),
        }
    }
    
    /// Get at-risk commitments
    pub fn get_at_risk_commitments(&self) -> Vec<&Commitment> {
        self.commitment_store.values()
            .filter(|c| {
                c.status == CommitmentStatus::Active &&
                self.is_at_risk(c)
            })
            .collect()
    }
    
    fn is_at_risk(&self, commitment: &Commitment) -> bool {
        if let Some(due) = commitment.due {
            let now = Timestamp::now();
            let days_remaining = (due.0 - now.0) as f64 / (86400.0 * 1e9);
            
            // Check if goal progress suggests we'll miss deadline
            if let Some(goal_id) = commitment.goal {
                if let Some(goal) = self.goal_store.get(&goal_id) {
                    let remaining_work = 1.0 - goal.progress.percentage;
                    let days_needed = if goal.progress.velocity > 0.0 {
                        remaining_work / goal.progress.velocity
                    } else {
                        f64::INFINITY
                    };
                    
                    return days_needed > days_remaining;
                }
            }
            
            // No goal, check if deadline is soon with no indication of progress
            return days_remaining < 7.0;
        }
        
        false
    }
}
```

## 6.5 Prophecy Queries

```rust
impl PlanningEngine {
    // ========================================================================
    // PROPHECY QUERIES
    // ========================================================================
    
    /// Scan for predicted blockers
    pub fn scan_blocker_prophecy(&self) -> Vec<BlockerProphecy> {
        let mut prophecies = Vec::new();
        
        for goal in self.get_active_goals() {
            let predicted = self.predict_blockers(goal);
            
            for blocker in predicted {
                prophecies.push(BlockerProphecy {
                    goal_id: goal.id,
                    predicted_blocker: blocker,
                    prediction_confidence: 0.7,  // Would calculate
                    days_until_materialization: 7.0,  // Would calculate
                    evidence: Vec::new(),
                    recommended_actions: Vec::new(),
                });
            }
        }
        
        prophecies
    }
    
    /// Listen for progress echoes
    pub fn listen_progress_echoes(&self) -> Vec<ProgressEcho> {
        let mut echoes = Vec::new();
        
        for goal in self.get_active_goals() {
            if goal.progress.percentage > 0.5 && goal.physics.momentum > 0.5 {
                // Strong progress, completion echo likely
                let eta_days = if let Some(eta) = goal.progress.eta {
                    let now = Timestamp::now();
                    (eta.0 - now.0) as f64 / (86400.0 * 1e9)
                } else {
                    30.0
                };
                
                if eta_days < 14.0 {
                    echoes.push(ProgressEcho {
                        goal_id: goal.id,
                        source_milestone: Milestone {
                            name: format!("{} completed", goal.title),
                            description: "Goal completion".to_string(),
                        },
                        echo_strength: goal.physics.momentum,
                        estimated_arrival: std::time::Duration::from_secs(
                            (eta_days * 86400.0) as u64
                        ),
                        carried_information: self.extract_echo_info(goal),
                        confidence: goal.feelings.confidence,
                    });
                }
            }
        }
        
        echoes
    }
    
    /// Get decision prophecy (preview consequences)
    pub fn get_decision_prophecy(
        &self,
        question: &str,
        options: &[DecisionPath],
    ) -> DecisionProphecy {
        let paths: Vec<_> = options.iter()
            .map(|option| ProphecyPath {
                path: option.clone(),
                timeline: self.project_path_timeline(option),
                final_state: self.project_path_final_state(option),
                risk_profile: self.assess_path_risk(option),
                opportunity_profile: self.assess_path_opportunity(option),
            })
            .collect();
        
        DecisionProphecy {
            question: DecisionQuestion {
                question: question.to_string(),
                context: String::new(),
                constraints: Vec::new(),
                asked_at: Timestamp::now(),
            },
            paths,
            confidence: 0.7,
            sources: Vec::new(),
            warnings: Vec::new(),
        }
    }
}
```

---

# SPEC-07: INDEXES

## 7.1 Index Structure

```rust
/// All indexes for fast lookup
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanIndexes {
    // ========================================================================
    // GOAL INDEXES
    // ========================================================================
    
    /// Goals by status
    pub goals_by_status: HashMap<GoalStatus, Vec<GoalId>>,
    
    /// Goals by priority
    pub goals_by_priority: HashMap<Priority, Vec<GoalId>>,
    
    /// Goals by deadline (sorted)
    pub goals_by_deadline: BTreeMap<Timestamp, Vec<GoalId>>,
    
    /// Goals by parent (for tree traversal)
    pub goals_by_parent: HashMap<GoalId, Vec<GoalId>>,
    
    /// Root goals (no parent)
    pub root_goals: Vec<GoalId>,
    
    /// Active goals (quick access)
    pub active_goals: Vec<GoalId>,
    
    /// Blocked goals
    pub blocked_goals: Vec<GoalId>,
    
    /// Goals by tag
    pub goals_by_tag: HashMap<String, Vec<GoalId>>,
    
    // ========================================================================
    // DECISION INDEXES
    // ========================================================================
    
    /// Decisions by goal
    pub decisions_by_goal: HashMap<GoalId, Vec<DecisionId>>,
    
    /// Decisions by time (sorted)
    pub decisions_by_time: BTreeMap<Timestamp, Vec<DecisionId>>,
    
    /// Pending decisions
    pub pending_decisions: Vec<DecisionId>,
    
    /// Regretted decisions
    pub regretted_decisions: Vec<DecisionId>,
    
    /// Decision chains (root -> descendants)
    pub decision_chains: HashMap<DecisionId, Vec<DecisionId>>,
    
    // ========================================================================
    // COMMITMENT INDEXES
    // ========================================================================
    
    /// Commitments by due date
    pub commitments_by_due: BTreeMap<Timestamp, Vec<CommitmentId>>,
    
    /// Commitments by stakeholder
    pub commitments_by_stakeholder: HashMap<StakeholderId, Vec<CommitmentId>>,
    
    /// Commitments by goal
    pub commitments_by_goal: HashMap<GoalId, Vec<CommitmentId>>,
    
    /// Active commitments
    pub active_commitments: Vec<CommitmentId>,
    
    /// At-risk commitments
    pub at_risk_commitments: Vec<CommitmentId>,
    
    // ========================================================================
    // URGENT ITEMS
    // ========================================================================
    
    /// Items due within 7 days
    pub urgent_items: Vec<UrgentItem>,
    
    // ========================================================================
    // RELATIONSHIP INDEXES
    // ========================================================================
    
    /// Goal relationships (for graph queries)
    pub goal_relationships: HashMap<GoalId, Vec<(GoalId, GoalRelationship)>>,
    
    /// Commitment entanglements
    pub commitment_entanglements: HashMap<CommitmentId, Vec<CommitmentId>>,
}
```

## 7.2 Index Operations

```rust
impl PlanIndexes {
    /// Create new empty indexes
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a goal to indexes
    pub fn add_goal(&mut self, goal: &Goal) {
        // By status
        self.goals_by_status
            .entry(goal.status)
            .or_default()
            .push(goal.id);
        
        // By priority
        self.goals_by_priority
            .entry(goal.priority)
            .or_default()
            .push(goal.id);
        
        // By deadline
        if let Some(deadline) = goal.deadline {
            self.goals_by_deadline
                .entry(deadline)
                .or_default()
                .push(goal.id);
        }
        
        // By parent
        if let Some(parent) = goal.parent {
            self.goals_by_parent
                .entry(parent)
                .or_default()
                .push(goal.id);
        } else {
            self.root_goals.push(goal.id);
        }
        
        // By tag
        for tag in &goal.tags {
            self.goals_by_tag
                .entry(tag.clone())
                .or_default()
                .push(goal.id);
        }
        
        // Active/blocked quick access
        if goal.status == GoalStatus::Active {
            self.active_goals.push(goal.id);
        } else if goal.status == GoalStatus::Blocked {
            self.blocked_goals.push(goal.id);
        }
        
        // Check urgency
        self.check_urgent_goal(goal);
    }
    
    /// Goal status changed
    pub fn goal_status_changed(&mut self, id: GoalId, old: GoalStatus, new: GoalStatus) {
        // Remove from old status
        if let Some(list) = self.goals_by_status.get_mut(&old) {
            list.retain(|&x| x != id);
        }
        
        // Add to new status
        self.goals_by_status
            .entry(new)
            .or_default()
            .push(id);
        
        // Update quick access lists
        match old {
            GoalStatus::Active => self.active_goals.retain(|&x| x != id),
            GoalStatus::Blocked => self.blocked_goals.retain(|&x| x != id),
            _ => {}
        }
        
        match new {
            GoalStatus::Active => self.active_goals.push(id),
            GoalStatus::Blocked => self.blocked_goals.push(id),
            _ => {}
        }
    }
    
    /// Add a decision to indexes
    pub fn add_decision(&mut self, decision: &Decision) {
        // By goal
        for goal_id in &decision.affected_goals {
            self.decisions_by_goal
                .entry(*goal_id)
                .or_default()
                .push(decision.id);
        }
        
        // By time
        if let Some(timestamp) = decision.crystallized_at {
            self.decisions_by_time
                .entry(timestamp)
                .or_default()
                .push(decision.id);
        }
        
        // Pending
        if decision.status == DecisionStatus::Pending {
            self.pending_decisions.push(decision.id);
        }
    }
    
    /// Decision crystallized
    pub fn decision_crystallized(&mut self, id: DecisionId) {
        self.pending_decisions.retain(|&x| x != id);
    }
    
    /// Add a commitment to indexes
    pub fn add_commitment(&mut self, commitment: &Commitment) {
        // By due date
        if let Some(due) = commitment.due {
            self.commitments_by_due
                .entry(due)
                .or_default()
                .push(commitment.id);
        }
        
        // By stakeholder
        self.commitments_by_stakeholder
            .entry(commitment.made_to.id)
            .or_default()
            .push(commitment.id);
        
        // By goal
        if let Some(goal_id) = commitment.goal {
            self.commitments_by_goal
                .entry(goal_id)
                .or_default()
                .push(commitment.id);
        }
        
        // Active
        if commitment.status == CommitmentStatus::Active {
            self.active_commitments.push(commitment.id);
        }
        
        // Check urgency
        self.check_urgent_commitment(commitment);
    }
    
    /// Check if goal should be in urgent list
    fn check_urgent_goal(&mut self, goal: &Goal) {
        if let Some(deadline) = goal.deadline {
            let now = Timestamp::now();
            let days = (deadline.0 - now.0) as f64 / (86400.0 * 1e9);
            
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
    
    /// Rebuild all indexes from scratch
    pub fn rebuild(
        &mut self,
        goals: &HashMap<GoalId, Goal>,
        decisions: &HashMap<DecisionId, Decision>,
        commitments: &HashMap<CommitmentId, Commitment>,
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
    }
}
```

---

# SPEC-08: VALIDATION

## 8.1 Validation Rules

```rust
/// Validation error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("Goal title is required")]
    GoalTitleRequired,
    
    #[error("Goal title too long (max {max}, got {got})")]
    GoalTitleTooLong { max: usize, got: usize },
    
    #[error("Goal intention is required")]
    IntentionRequired,
    
    #[error("Invalid priority: {0:?}")]
    InvalidPriority(String),
    
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition { from: GoalStatus, to: GoalStatus },
    
    #[error("Deadline must be in the future")]
    DeadlineInPast,
    
    #[error("Progress must be between 0 and 1")]
    InvalidProgress,
    
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<GoalId>),
    
    #[error("Parent goal not found: {0:?}")]
    ParentNotFound(GoalId),
    
    #[error("Decision question is required")]
    DecisionQuestionRequired,
    
    #[error("Decision must have at least 2 options")]
    InsufficientOptions,
    
    #[error("Commitment promise is required")]
    PromiseRequired,
    
    #[error("Commitment stakeholder is required")]
    StakeholderRequired,
    
    #[error("Weight must be between 0 and 1")]
    InvalidWeight,
}

/// Validation result
pub type ValidationResult<T> = Result<T, Vec<ValidationError>>;
```

## 8.2 Goal Validation

```rust
impl PlanningEngine {
    /// Validate goal creation request
    pub fn validate_create_goal(&self, request: &CreateGoalRequest) -> ValidationResult<()> {
        let mut errors = Vec::new();
        
        // Title required
        if request.title.trim().is_empty() {
            errors.push(ValidationError::GoalTitleRequired);
        }
        
        // Title length
        if request.title.len() > 200 {
            errors.push(ValidationError::GoalTitleTooLong {
                max: 200,
                got: request.title.len(),
            });
        }
        
        // Intention required
        if request.intention.trim().is_empty() {
            errors.push(ValidationError::IntentionRequired);
        }
        
        // Deadline in future
        if let Some(deadline) = request.deadline {
            if deadline < Timestamp::now() {
                errors.push(ValidationError::DeadlineInPast);
            }
        }
        
        // Parent exists
        if let Some(parent_id) = request.parent {
            if !self.goal_store.contains_key(&parent_id) {
                errors.push(ValidationError::ParentNotFound(parent_id));
            }
        }
        
        // Check circular dependencies
        if let Some(deps) = &request.dependencies {
            if let Some(cycle) = self.detect_dependency_cycle(deps, &[]) {
                errors.push(ValidationError::CircularDependency(cycle));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate status transition
    pub fn validate_status_transition(
        &self,
        current: GoalStatus,
        target: GoalStatus,
    ) -> ValidationResult<()> {
        let valid = match (current, target) {
            // From Draft
            (GoalStatus::Draft, GoalStatus::Active) => true,
            (GoalStatus::Draft, GoalStatus::Abandoned) => true,
            
            // From Active
            (GoalStatus::Active, GoalStatus::Blocked) => true,
            (GoalStatus::Active, GoalStatus::Paused) => true,
            (GoalStatus::Active, GoalStatus::Completed) => true,
            (GoalStatus::Active, GoalStatus::Abandoned) => true,
            
            // From Blocked
            (GoalStatus::Blocked, GoalStatus::Active) => true,
            (GoalStatus::Blocked, GoalStatus::Abandoned) => true,
            
            // From Paused
            (GoalStatus::Paused, GoalStatus::Active) => true,
            (GoalStatus::Paused, GoalStatus::Abandoned) => true,
            
            // From Completed/Abandoned (reincarnation only)
            (GoalStatus::Completed, GoalStatus::Reborn) => true,
            (GoalStatus::Abandoned, GoalStatus::Reborn) => true,
            
            // From Reborn
            (GoalStatus::Reborn, GoalStatus::Active) => true,
            
            _ => false,
        };
        
        if valid {
            Ok(())
        } else {
            Err(vec![ValidationError::InvalidStatusTransition {
                from: current,
                to: target,
            }])
        }
    }
    
    /// Detect circular dependencies
    fn detect_dependency_cycle(
        &self,
        deps: &[GoalId],
        visited: &[GoalId],
    ) -> Option<Vec<GoalId>> {
        for dep_id in deps {
            if visited.contains(dep_id) {
                return Some(visited.to_vec());
            }
            
            if let Some(dep_goal) = self.goal_store.get(dep_id) {
                let mut new_visited = visited.to_vec();
                new_visited.push(*dep_id);
                
                if let Some(cycle) = self.detect_dependency_cycle(
                    &dep_goal.dependencies,
                    &new_visited,
                ) {
                    return Some(cycle);
                }
            }
        }
        
        None
    }
}
```

## 8.3 Decision Validation

```rust
impl PlanningEngine {
    /// Validate decision creation
    pub fn validate_create_decision(
        &self,
        request: &CreateDecisionRequest,
    ) -> ValidationResult<()> {
        let mut errors = Vec::new();
        
        // Question required
        if request.question.trim().is_empty() {
            errors.push(ValidationError::DecisionQuestionRequired);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate crystallization
    pub fn validate_crystallize(
        &self,
        decision: &Decision,
    ) -> ValidationResult<()> {
        let mut errors = Vec::new();
        
        // Must have at least 2 options
        if decision.shadows.len() < 2 {
            errors.push(ValidationError::InsufficientOptions);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## 8.4 Commitment Validation

```rust
impl PlanningEngine {
    /// Validate commitment creation
    pub fn validate_create_commitment(
        &self,
        request: &CreateCommitmentRequest,
    ) -> ValidationResult<()> {
        let mut errors = Vec::new();
        
        // Promise required
        if request.promise.description.trim().is_empty() {
            errors.push(ValidationError::PromiseRequired);
        }
        
        // Stakeholder required
        if request.stakeholder.name.trim().is_empty() {
            errors.push(ValidationError::StakeholderRequired);
        }
        
        // Weight valid
        if let Some(weight) = self.calculate_commitment_weight_preview(request) {
            if !(0.0..=1.0).contains(&weight) {
                errors.push(ValidationError::InvalidWeight);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## 8.5 MCP Input Validation

```rust
/// MCP parameter validators
pub mod validators {
    use super::*;
    
    /// Validate goal_id parameter
    pub fn validate_goal_id(value: &serde_json::Value) -> Result<GoalId, String> {
        let s = value.as_str()
            .ok_or("goal_id must be a string")?;
        
        let uuid = Uuid::parse_str(s)
            .map_err(|_| "goal_id must be a valid UUID")?;
        
        Ok(GoalId(uuid))
    }
    
    /// Validate progress parameter
    pub fn validate_progress(value: &serde_json::Value) -> Result<f64, String> {
        let n = value.as_f64()
            .ok_or("progress must be a number")?;
        
        if !(0.0..=1.0).contains(&n) {
            return Err("progress must be between 0 and 1".to_string());
        }
        
        Ok(n)
    }
    
    /// Validate priority parameter
    pub fn validate_priority(value: &serde_json::Value) -> Result<Priority, String> {
        let s = value.as_str()
            .ok_or("priority must be a string")?;
        
        match s.to_lowercase().as_str() {
            "critical" => Ok(Priority::Critical),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            "someday" => Ok(Priority::Someday),
            _ => Err(format!("invalid priority: {}", s)),
        }
    }
    
    /// Validate timestamp parameter
    pub fn validate_timestamp(value: &serde_json::Value) -> Result<Timestamp, String> {
        if let Some(n) = value.as_i64() {
            return Ok(Timestamp(n));
        }
        
        if let Some(s) = value.as_str() {
            // Try ISO 8601
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Ok(Timestamp(dt.timestamp_nanos_opt().unwrap_or(0)));
            }
        }
        
        Err("timestamp must be nanos (i64) or ISO 8601 string".to_string())
    }
}
```

---

## Part 2 Complete

**Covered:**
- SPEC-05: Write Engine
- SPEC-06: Query Engine
- SPEC-07: Indexes
- SPEC-08: Validation

**Next (Part 3):**
- SPEC-09: CLI
- SPEC-10: MCP Server
- SPEC-11: Sister Integration
- SPEC-12: Tests

---

*Document: AGENTIC-PLANNING-SPEC-PART2.md*
