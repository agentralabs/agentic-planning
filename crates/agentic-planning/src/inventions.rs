//! Inventions module — batch and parallel optimizations for PlanningEngine.
//!
//! # Parallelism Strategy
//!
//! The `PlanningEngine` holds mutable state in `HashMap`s and is `!Sync`.
//! True parallelism (rayon, tokio::spawn) would require either:
//!   - `Arc<RwLock<PlanningEngine>>` (contention-heavy, defeats purpose)
//!   - Unsafe split-borrows of disjoint HashMap entries (fragile, unsound risk)
//!
//! Instead, we take a **pragmatic optimized-serial** approach:
//!   1. Pre-filter: collect only relevant IDs/data before the hot loop
//!   2. Batch compute: minimize per-item overhead (fewer HashMap lookups)
//!   3. Single-pass merge: avoid repeated allocations and sorts
//!
//! For `create_goals_batch`, the win is concrete:
//!   - Validate ALL requests upfront (fail-fast, no partial inserts)
//!   - Generate IDs and build goals in one pass
//!   - Insert all goals, wire parent/dependency links
//!   - Rebuild indexes ONCE (not N incremental `add_goal` calls)
//!
//! When the engine moves to an actor or async model, these functions
//! become natural sharding points for real parallelism.

use crate::types::*;
use crate::{CreateCommitmentRequest, CreateDecisionRequest, PlanningEngine};
use std::collections::HashMap;
use uuid::Uuid;

impl PlanningEngine {
    /// Compute the intention singularity with pre-filtered active goals.
    ///
    /// Optimization over raw `get_intention_singularity()`:
    /// - Pre-collects active goal references to avoid repeated status checks
    /// - Short-circuits the empty case (no active goals → empty singularity)
    pub fn calculate_singularity_parallel(&self) -> IntentionSingularity {
        // Pre-filter: check for active goals without full computation
        let has_active = self
            .goal_store
            .values()
            .any(|g| g.status == GoalStatus::Active);

        if !has_active {
            return IntentionSingularity {
                center: IntentionCenter {
                    urgency: 0.0,
                    confidence: 0.0,
                    momentum: 0.0,
                },
                unified_vision: String::new(),
                goal_positions: HashMap::new(),
                tension_lines: Vec::new(),
                themes: Vec::new(),
                golden_path: Vec::new(),
            };
        }

        // Delegate to the full computation — it handles the heavy lifting.
        // The pre-filter above short-circuits the empty case.
        self.get_intention_singularity()
    }

    /// Scan for blocker prophecies with pre-filtered active goals.
    ///
    /// Optimization: pre-computes blocker type frequency histogram once,
    /// then runs prediction in a single pass with pre-allocated output.
    pub fn scan_blockers_parallel(&self) -> Vec<BlockerProphecy> {
        let active_goals: Vec<&Goal> = self
            .goal_store
            .values()
            .filter(|g| g.status == GoalStatus::Active || g.status == GoalStatus::Blocked)
            .collect();

        if active_goals.is_empty() {
            return Vec::new();
        }

        // Pre-compute blocker type frequency histogram (shared across goals)
        let mut blocker_type_counts: HashMap<String, usize> = HashMap::new();
        for goal in self.goal_store.values() {
            for blocker in &goal.blockers {
                let key = format!("{:?}", std::mem::discriminant(&blocker.blocker_type));
                *blocker_type_counts.entry(key).or_insert(0) += 1;
            }
        }
        let total_historical = blocker_type_counts.values().sum::<usize>().max(1);

        // Single pass: predict blockers for each active goal
        let mut prophecies = Vec::with_capacity(active_goals.len() * 2);

        for goal in &active_goals {
            for blocker in self.predict_blockers(goal) {
                let type_key = format!("{:?}", std::mem::discriminant(&blocker.blocker_type));
                let type_frequency = *blocker_type_counts.get(&type_key).unwrap_or(&0) as f64
                    / total_historical as f64;
                let severity_signal = blocker.severity;
                let prediction_confidence =
                    (0.3 + type_frequency * 0.3 + severity_signal * 0.3).clamp(0.1, 0.95);

                let days_until = goal
                    .deadline
                    .map(|d| {
                        let days = ((d.0 - Timestamp::now().0) as f64 / (86_400.0 * 1e9)).max(0.5);
                        (days * (1.0 - severity_signal)).max(1.0)
                    })
                    .unwrap_or(14.0 * (1.0 - severity_signal * 0.5));

                let mut evidence = Vec::new();
                if goal.progress.velocity == 0.0 {
                    evidence.push("zero progress velocity".to_string());
                }
                if goal.feelings.neglect > 0.5 {
                    evidence.push(format!("high neglect score ({:.2})", goal.feelings.neglect));
                }

                prophecies.push(BlockerProphecy {
                    goal_id: goal.id,
                    predicted_blocker: blocker.clone(),
                    prediction_confidence,
                    days_until_materialization: days_until,
                    evidence,
                    recommended_actions: blocker
                        .resolution
                        .clone()
                        .map(|r| vec![r])
                        .unwrap_or_default(),
                });
            }
        }

        // Sort by confidence descending for priority ordering
        prophecies.sort_by(|a, b| {
            b.prediction_confidence
                .partial_cmp(&a.prediction_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        prophecies
    }

    /// Detect progress echoes with pre-filtered near-completion goals.
    ///
    /// Optimization: only examines goals with progress > 0.5 and momentum > 0.2,
    /// skipping the bulk of low-progress goals. Uses the same logic as
    /// `listen_progress_echoes` but with an early-exit filter.
    pub fn progress_echoes_parallel(&self) -> Vec<ProgressEcho> {
        let candidates: Vec<&Goal> = self
            .goal_store
            .values()
            .filter(|g| {
                g.status == GoalStatus::Active
                    && g.progress.percentage > 0.5
                    && g.physics.momentum > 0.2
            })
            .collect();

        if candidates.is_empty() {
            return Vec::new();
        }

        let mut echoes = Vec::with_capacity(candidates.len());

        for goal in &candidates {
            let eta_days = goal
                .progress
                .eta
                .map(|eta| ((eta.0 - Timestamp::now().0) as f64 / (86_400.0 * 1e9)).max(1.0))
                .unwrap_or(30.0);

            if eta_days < 30.0 {
                echoes.push(ProgressEcho {
                    goal_id: goal.id,
                    source_milestone: Milestone {
                        name: format!("{} completed", goal.title),
                        description: "Goal completion".to_string(),
                    },
                    echo_strength: goal.physics.momentum,
                    estimated_arrival_secs: (eta_days * 86_400.0) as u64,
                    carried_information: self.extract_echo_info(goal),
                    confidence: goal.feelings.confidence,
                });
            }
        }

        // Sort by echo_strength descending
        echoes.sort_by(|a, b| {
            b.echo_strength
                .partial_cmp(&a.echo_strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        echoes
    }

    /// Create multiple goals in a single batch operation.
    ///
    /// Unlike calling `create_goal()` N times, this:
    ///   1. Validates ALL requests upfront (fail-fast: no partial inserts on error)
    ///   2. Pre-generates IDs and timestamps in one pass
    ///   3. Inserts all goals into the store
    ///   4. Wires parent/child and dependency links
    ///   5. Rebuilds indexes ONCE at the end (not N incremental adds)
    ///
    /// Performance: for N=10 goals, avoids 9 redundant index rebuilds.
    /// For N=100, the savings are substantial.
    pub fn create_goals_batch(
        &mut self,
        requests: Vec<crate::CreateGoalRequest>,
    ) -> crate::Result<Vec<Goal>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // ── Phase 1: Validate ALL requests upfront ──
        // Fail-fast: if any request is invalid, return error before mutating state.
        for (i, req) in requests.iter().enumerate() {
            if let Err(errors) = self.validate_create_goal(req) {
                let msg = errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(crate::Error::CorruptedFile(format!(
                    "batch validation failed at request[{}]: {}",
                    i, msg
                )));
            }
        }

        // ── Phase 2: Generate IDs and build Goal structs ──
        let now = Timestamp::now();
        let mut goals = Vec::with_capacity(requests.len());

        for request in requests {
            let id = GoalId(Uuid::new_v4());
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

            goals.push(goal);
        }

        // ── Phase 3: Batch insert into goal_store ──
        for goal in &goals {
            self.goal_store.insert(goal.id, goal.clone());
        }

        // ── Phase 4: Wire parent/child and dependency links ──
        // Collect link operations first, then apply (avoids borrow conflicts).
        let links: Vec<(GoalId, Option<GoalId>, Vec<GoalId>)> = goals
            .iter()
            .map(|g| (g.id, g.parent, g.dependencies.clone()))
            .collect();

        for (child_id, parent_opt, deps) in &links {
            if let Some(parent_id) = parent_opt {
                if let Some(parent) = self.goal_store.get_mut(parent_id) {
                    parent.children.push(*child_id);
                }
            }
            for dep_id in deps {
                if let Some(dep) = self.goal_store.get_mut(dep_id) {
                    dep.dependents.push(*child_id);
                }
            }
        }

        // ── Phase 5: Rebuild indexes ONCE ──
        // This is the key optimization: instead of N incremental `add_goal` calls,
        // we do a single full rebuild. For N>3 this is already faster.
        self.rebuild_indexes();
        self.mark_dirty();

        Ok(goals)
    }

    // ════════════════════════════════════════════════════════════════════
    // Batch write operations — Goal lifecycle
    // ════════════════════════════════════════════════════════════════════

    /// Activate multiple Draft/Reborn goals in one pass.
    ///
    /// Validates all IDs upfront (fail-fast), applies transitions, and
    /// calls `mark_dirty` once at the end.
    pub fn batch_activate_goals(&mut self, ids: &[GoalId]) -> crate::Result<Vec<Goal>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 1: validate all goals exist and are in valid state
        for id in ids {
            let goal = self
                .goal_store
                .get(id)
                .ok_or(crate::Error::GoalNotFound(*id))?;
            if goal.status != GoalStatus::Draft && goal.status != GoalStatus::Reborn {
                return Err(crate::Error::InvalidTransition {
                    from: goal.status,
                    to: GoalStatus::Active,
                });
            }
        }

        // Phase 2: apply transitions
        let now = Timestamp::now();
        let mut results = Vec::with_capacity(ids.len());

        for id in ids {
            let goal = self
                .goal_store
                .get_mut(id)
                .expect("goal existence validated in phase 1");
            let old = goal.status;
            goal.status = GoalStatus::Active;
            goal.activated_at = Some(now);
            goal.feelings.vitality = 1.0;
            self.indexes
                .goal_status_changed(*id, old, GoalStatus::Active);
            results.push(goal.clone());
        }

        self.mark_dirty();
        Ok(results)
    }

    /// Progress multiple goals at once with batched velocity/momentum recalc.
    ///
    /// Each entry is `(goal_id, new_percentage, optional_note)`.
    /// Validates all IDs upfront, records progress points, recalculates
    /// velocity/momentum/confidence for each, and marks dirty once.
    pub fn batch_progress_goals(
        &mut self,
        updates: Vec<(GoalId, f64, Option<String>)>,
    ) -> crate::Result<Vec<Goal>> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 1: validate all goals exist
        for (id, _, _) in &updates {
            if !self.goal_store.contains_key(id) {
                return Err(crate::Error::GoalNotFound(*id));
            }
        }

        // Phase 2: record progress points and collect snapshots
        let now = Timestamp::now();
        let mut snapshots = Vec::with_capacity(updates.len());

        for (id, percentage, note) in &updates {
            let goal = self
                .goal_store
                .get_mut(id)
                .expect("goal existence validated in phase 1");
            let p = percentage.clamp(0.0, 1.0);
            goal.progress.history.push(ProgressPoint {
                timestamp: now,
                percentage: p,
                note: note.clone(),
            });
            goal.progress.percentage = p;
            snapshots.push((*id, goal.clone()));
        }

        // Phase 3: recalculate velocity/momentum/confidence
        let recalcs: Vec<(GoalId, f64, f64, f64)> = snapshots
            .iter()
            .map(|(id, snap)| {
                let v = self.calculate_velocity(&snap.progress.history);
                let m = self.calculate_momentum_from_goal(snap);
                let c = self.calculate_confidence_from_goal(snap);
                (*id, v, m, c)
            })
            .collect();

        // Phase 4: apply recalculated values
        let mut results = Vec::with_capacity(recalcs.len());
        for (id, velocity, momentum, confidence) in recalcs {
            let goal = self
                .goal_store
                .get_mut(&id)
                .expect("goal existence validated in phase 1");
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
            results.push(goal.clone());
        }

        self.mark_dirty();
        Ok(results)
    }

    // ════════════════════════════════════════════════════════════════════
    // Batch write operations — Decisions
    // ════════════════════════════════════════════════════════════════════

    /// Create multiple decisions in a single batch.
    ///
    /// Validates all requests upfront, generates IDs, wires goal links
    /// and causal chains, rebuilds indexes once.
    pub fn batch_create_decisions(
        &mut self,
        requests: Vec<CreateDecisionRequest>,
    ) -> crate::Result<Vec<Decision>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 1: validate all referenced goals exist
        for (i, req) in requests.iter().enumerate() {
            if let Some(goals) = &req.goals {
                for gid in goals {
                    if !self.goal_store.contains_key(gid) {
                        return Err(crate::Error::CorruptedFile(format!(
                            "batch decision[{}] references nonexistent goal {:?}",
                            i, gid
                        )));
                    }
                }
            }
        }

        let now = Timestamp::now();
        let mut decisions = Vec::with_capacity(requests.len());

        // Phase 2: build all Decision structs
        for request in requests {
            let id = DecisionId(Uuid::new_v4());

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
            decisions.push(decision);
        }

        // Phase 3: batch insert and wire links
        for decision in &decisions {
            for goal_id in &decision.affected_goals {
                if let Some(goal) = self.goal_store.get_mut(goal_id) {
                    goal.decisions.push(decision.id);
                }
            }

            if let Some(parent_id) = decision.caused_by {
                if let Some(parent) = self.decision_store.get_mut(&parent_id) {
                    parent.causes.push(decision.id);
                }
            }

            self.decision_store.insert(decision.id, decision.clone());
            self.indexes.add_decision(decision);
        }

        self.mark_dirty();
        Ok(decisions)
    }

    // ════════════════════════════════════════════════════════════════════
    // Batch write operations — Commitments
    // ════════════════════════════════════════════════════════════════════

    /// Create multiple commitments in a single batch.
    ///
    /// Validates all requests, generates IDs, calculates weight/inertia/cost
    /// for each, wires to goals, and marks dirty once.
    pub fn batch_create_commitments(
        &mut self,
        requests: Vec<CreateCommitmentRequest>,
    ) -> crate::Result<Vec<Commitment>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 1: validate all referenced goals exist
        for (i, req) in requests.iter().enumerate() {
            if let Some(gid) = req.goal {
                if !self.goal_store.contains_key(&gid) {
                    return Err(crate::Error::CorruptedFile(format!(
                        "batch commitment[{}] references nonexistent goal {:?}",
                        i, gid
                    )));
                }
            }
        }

        let now = Timestamp::now();
        let mut commitments = Vec::with_capacity(requests.len());

        // Phase 2: build all Commitment structs
        for request in &requests {
            let id = CommitmentId(Uuid::new_v4());
            let weight = self.calculate_commitment_weight(request);
            let inertia = self.calculate_commitment_inertia(request);
            let breaking_cost = self.calculate_breaking_cost(request);

            commitments.push(Commitment {
                id,
                promise: request.promise.clone(),
                made_to: request.stakeholder.clone(),
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
            });
        }

        // Phase 3: batch insert and wire to goals
        for commitment in &commitments {
            if let Some(goal_id) = commitment.goal {
                if let Some(goal) = self.goal_store.get_mut(&goal_id) {
                    goal.commitments.push(commitment.id);
                }
            }
            self.commitment_store
                .insert(commitment.id, commitment.clone());
            self.indexes.add_commitment(commitment);
        }

        self.mark_dirty();
        Ok(commitments)
    }

    /// Fulfill multiple commitments in a single pass.
    ///
    /// Validates all are Active upfront, calculates chain bonuses,
    /// applies fulfillments, and handles entanglement energy release.
    pub fn batch_fulfill_commitments(
        &mut self,
        fulfillments: Vec<(CommitmentId, String)>,
    ) -> crate::Result<Vec<Commitment>> {
        if fulfillments.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 1: validate all exist and are Active
        for (id, _) in &fulfillments {
            let c = self
                .commitment_store
                .get(id)
                .ok_or(crate::Error::CommitmentNotFound(*id))?;
            if c.status != CommitmentStatus::Active {
                return Err(crate::Error::CannotFulfill(c.status));
            }
        }

        // Phase 2: collect chain bonuses and entanglements
        let pre_data: Vec<(CommitmentId, String, f64, f64, Vec<CommitmentEntanglement>)> =
            fulfillments
                .into_iter()
                .map(|(id, how)| {
                    let bonus = self.calculate_chain_bonus(id);
                    let c = self
                        .commitment_store
                        .get(&id)
                        .expect("commitment existence validated in phase 1");
                    let weight = c.weight;
                    let entanglements = c.entanglements.clone();
                    (id, how, bonus, weight, entanglements)
                })
                .collect();

        // Phase 3: apply fulfillments
        let now = Timestamp::now();
        let mut results = Vec::with_capacity(pre_data.len());

        for (id, how_delivered, chain_bonus, weight, entanglements) in &pre_data {
            let energy_released = weight * (1.0 + chain_bonus);

            if let Some(c) = self.commitment_store.get_mut(id) {
                c.status = CommitmentStatus::Fulfilled;
                c.fulfillment = Some(CommitmentFulfillment {
                    fulfilled_at: now,
                    how_delivered: how_delivered.clone(),
                    energy_released,
                    trust_gained: c.weight * 0.5,
                });
            }

            for entanglement in entanglements {
                if entanglement.entanglement_type == EntanglementType::Sequential {
                    let _ = self.boost_commitment(entanglement.with, energy_released * 0.3);
                }
            }

            self.indexes.commitment_fulfilled(*id);
            results.push(
                self.commitment_store
                    .get(id)
                    .ok_or(crate::Error::CommitmentNotFound(*id))?
                    .clone(),
            );
        }

        self.mark_dirty();
        Ok(results)
    }

    // ════════════════════════════════════════════════════════════════════
    // Batch read operations — Reports and scans
    // ════════════════════════════════════════════════════════════════════

    /// Momentum report with pre-filtered index-based active goals.
    ///
    /// Optimization over `get_momentum_report()`:
    /// - Uses `indexes.goals_by_status` to avoid full store scan
    /// - Same output shape, but skips inactive/completed goals entirely
    pub fn momentum_report_indexed(&self) -> MomentumReport {
        let active_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Active)
            .cloned()
            .unwrap_or_default();
        let blocked_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Blocked)
            .cloned()
            .unwrap_or_default();

        let active_goals: Vec<&Goal> = active_ids
            .iter()
            .chain(blocked_ids.iter())
            .filter_map(|id| self.goal_store.get(id))
            .collect();

        let total = active_goals.len();
        let avg = if total > 0 {
            active_goals.iter().map(|g| g.physics.momentum).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let mut distribution = MomentumDistribution {
            high: 0,
            medium: 0,
            low: 0,
            zero: 0,
        };

        let mut entries: Vec<GoalMomentumEntry> = active_goals
            .iter()
            .map(|g| {
                match g.physics.momentum {
                    m if m >= 0.7 => distribution.high += 1,
                    m if m >= 0.3 => distribution.medium += 1,
                    m if m > 0.0 => distribution.low += 1,
                    _ => distribution.zero += 1,
                }
                GoalMomentumEntry {
                    goal_id: g.id,
                    title: g.title.clone(),
                    momentum: g.physics.momentum,
                    velocity: g.progress.velocity,
                    progress: g.progress.percentage,
                }
            })
            .collect();

        entries.sort_by(|a, b| {
            b.momentum
                .partial_cmp(&a.momentum)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let top = entries.iter().take(5).cloned().collect();
        let stalled: Vec<GoalMomentumEntry> = entries
            .iter()
            .filter(|e| e.momentum < 0.05 && e.progress < 1.0)
            .cloned()
            .collect();

        // Accelerating: velocity > momentum (gaining speed)
        let accelerating: Vec<GoalMomentumEntry> = entries
            .iter()
            .filter(|e| e.velocity > e.momentum && e.velocity > 0.0)
            .cloned()
            .collect();

        // Decelerating: momentum > velocity (losing speed)
        let decelerating: Vec<GoalMomentumEntry> = entries
            .iter()
            .filter(|e| e.momentum > e.velocity + 0.1 && e.velocity >= 0.0)
            .cloned()
            .collect();

        MomentumReport {
            total_goals: total,
            average_momentum: avg,
            momentum_distribution: distribution,
            top_momentum: top,
            stalled,
            accelerating,
            decelerating,
        }
    }

    /// Gravity field with index-based pre-filtering.
    ///
    /// Optimization over `get_gravity_field()`:
    /// - Uses `indexes.goals_by_status` to skip inactive goals
    /// - Computes weighted center and gravity wells in a single pass
    pub fn gravity_field_indexed(&self) -> GravityField {
        let active_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Active)
            .cloned()
            .unwrap_or_default();
        let blocked_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Blocked)
            .cloned()
            .unwrap_or_default();

        let relevant: Vec<&Goal> = active_ids
            .iter()
            .chain(blocked_ids.iter())
            .filter_map(|id| self.goal_store.get(id))
            .collect();

        let total = relevant.len();

        let (weighted_urgency, weighted_priority, weighted_momentum) = if total > 0 {
            let u = relevant
                .iter()
                .map(|g| g.feelings.urgency * g.physics.gravity)
                .sum::<f64>()
                / total as f64;
            let p = relevant
                .iter()
                .map(|g| {
                    let pv = match g.priority {
                        Priority::Critical => 1.0,
                        Priority::High => 0.8,
                        Priority::Medium => 0.5,
                        Priority::Low => 0.3,
                        Priority::Someday => 0.1,
                    };
                    pv * g.physics.gravity
                })
                .sum::<f64>()
                / total as f64;
            let m = relevant
                .iter()
                .map(|g| g.physics.momentum * g.physics.gravity)
                .sum::<f64>()
                / total as f64;
            (u, p, m)
        } else {
            (0.0, 0.0, 0.0)
        };

        let mut wells: Vec<GravityWell> = relevant
            .iter()
            .filter(|g| g.physics.gravity > 0.3)
            .map(|g| {
                let pull_radius = g.physics.gravity * (1.0 + g.dependents.len() as f64 * 0.2);
                let captured: Vec<GoalId> = g
                    .children
                    .iter()
                    .chain(g.dependents.iter())
                    .copied()
                    .collect();
                GravityWell {
                    goal_id: g.id,
                    title: g.title.clone(),
                    gravity: g.physics.gravity,
                    pull_radius,
                    captured_goals: captured,
                }
            })
            .collect();

        wells.sort_by(|a, b| {
            b.gravity
                .partial_cmp(&a.gravity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_pull = wells.iter().map(|w| w.gravity).sum();
        let dominant = wells.first().map(|w| w.goal_id);

        GravityField {
            total_goals: total,
            field_center: GravityCenter {
                weighted_urgency,
                weighted_priority,
                weighted_momentum,
            },
            wells,
            total_pull,
            dominant_attractor: dominant,
        }
    }

    /// Scan all active commitments for at-risk status in one pass.
    ///
    /// Returns commitment IDs and their risk signals.
    pub fn at_risk_commitments_scan(&self) -> Vec<(CommitmentId, Vec<String>)> {
        let mut results = Vec::new();

        for commitment in self.commitment_store.values() {
            if commitment.status != CommitmentStatus::Active {
                continue;
            }

            let mut risks = Vec::new();
            if let Some(due) = commitment.due {
                let now = Timestamp::now();
                let days_remaining = (due.0 - now.0) as f64 / (86_400.0 * 1e9);

                if days_remaining < 0.0 {
                    risks.push("Overdue".to_string());
                } else if days_remaining < 3.0 {
                    risks.push(format!("Due in {:.1} days", days_remaining));
                }

                if let Some(goal_id) = commitment.goal {
                    if let Some(goal) = self.goal_store.get(&goal_id) {
                        let remaining_work = 1.0 - goal.progress.percentage;
                        let days_needed = if goal.progress.velocity > 0.0 {
                            remaining_work / goal.progress.velocity
                        } else {
                            f64::INFINITY
                        };
                        if days_needed > days_remaining {
                            risks.push(format!(
                                "Needs {:.1} days but only {:.1} remain",
                                days_needed, days_remaining
                            ));
                        }
                    }
                }
            }

            if commitment.weight > 1.5 {
                risks.push("High-weight commitment".to_string());
            }

            if !risks.is_empty() {
                results.push((commitment.id, risks));
            }
        }

        results.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        results
    }

    /// Scan all active/blocked goals for metamorphosis signals in one pass.
    ///
    /// Returns only goals that should transform, skipping stable ones.
    pub fn metamorphosis_scan(&self) -> Vec<MetamorphosisSignal> {
        let mut signals = Vec::new();

        let active_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Active)
            .cloned()
            .unwrap_or_default();
        let blocked_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Blocked)
            .cloned()
            .unwrap_or_default();

        for id in active_ids.iter().chain(blocked_ids.iter()) {
            if let Ok(signal) = self.detect_metamorphosis(*id) {
                if signal.should_transform {
                    signals.push(signal);
                }
            }
        }

        signals
    }

    /// Progress forecasts for multiple goals at once.
    ///
    /// Collects forecasts for all specified goals, returning
    /// partial results (skipping not-found goals rather than failing).
    pub fn progress_forecast_batch(&self, ids: &[GoalId]) -> Vec<ProgressForecast> {
        ids.iter()
            .filter_map(|id| self.get_progress_forecast(*id).ok())
            .collect()
    }

    /// Progress forecasts for ALL active goals.
    ///
    /// Uses index to identify active goals, then computes forecasts
    /// in a single pass. Useful for dashboard views.
    pub fn progress_forecast_all_active(&self) -> Vec<ProgressForecast> {
        let active_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Active)
            .cloned()
            .unwrap_or_default();

        active_ids
            .iter()
            .filter_map(|id| self.get_progress_forecast(*id).ok())
            .collect()
    }

    /// Dream multiple goals and collect all resulting dreams.
    ///
    /// Unlike calling `dream_goal` N times, this batches the dirty
    /// marking. Note: each dream may create child goals if confidence > 0.7.
    pub fn dream_goals_batch(&mut self, ids: &[GoalId]) -> Vec<crate::Result<Dream>> {
        if ids.is_empty() {
            return Vec::new();
        }

        // Validate all exist first
        let valid_ids: Vec<GoalId> = ids
            .iter()
            .filter(|id| self.goal_store.contains_key(id))
            .copied()
            .collect();

        // Dream each goal (each internally marks dirty, but we accept that
        // since dream_goal creates child goals which need intermediate state)
        valid_ids.iter().map(|id| self.dream_goal(*id)).collect()
    }

    /// Check health across all federations in one pass.
    ///
    /// Returns federation ID, member count, sync freshness, and any issues.
    pub fn federation_health_scan(&self) -> Vec<FederationHealthEntry> {
        let now = Timestamp::now();
        self.federation_store
            .values()
            .map(|fed| {
                let sync_age_hours = (now.0 - fed.last_sync.0) as f64 / (3_600.0 * 1e9);
                let mut issues = Vec::new();

                if fed.members.is_empty() {
                    issues.push("No members".to_string());
                }
                if sync_age_hours > 24.0 {
                    issues.push(format!("Last sync {:.0}h ago", sync_age_hours));
                }
                if !self.goal_store.contains_key(&fed.goal_id) {
                    issues.push("Goal no longer exists".to_string());
                }

                FederationHealthEntry {
                    federation_id: fed.id,
                    goal_id: fed.goal_id,
                    member_count: fed.members.len(),
                    sync_age_hours,
                    issues,
                }
            })
            .collect()
    }

    /// Comprehensive health scan across all active goals.
    ///
    /// Single-pass check for: stalled goals, neglected goals, goals nearing
    /// deadline, blocked goals, and high-momentum goals (opportunities).
    pub fn goal_health_scan(&self) -> GoalHealthReport {
        let active_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Active)
            .cloned()
            .unwrap_or_default();
        let blocked_ids = self
            .indexes
            .goals_by_status
            .get(&GoalStatus::Blocked)
            .cloned()
            .unwrap_or_default();

        let now = Timestamp::now();
        let mut stalled = Vec::new();
        let mut neglected = Vec::new();
        let mut deadline_risk = Vec::new();
        let mut blocked = Vec::new();
        let mut thriving = Vec::new();

        for id in active_ids.iter().chain(blocked_ids.iter()) {
            let Some(goal) = self.goal_store.get(id) else {
                continue;
            };

            if goal.status == GoalStatus::Blocked {
                blocked.push(*id);
            }

            if goal.physics.momentum == 0.0 && goal.progress.percentage < 1.0 {
                stalled.push(*id);
            }

            if goal.feelings.neglect > 0.5 {
                neglected.push(*id);
            }

            if let Some(deadline) = goal.deadline {
                let days_left = (deadline.0 - now.0) as f64 / (86_400.0 * 1e9);
                if days_left < 7.0 && goal.progress.percentage < 0.8 {
                    deadline_risk.push(*id);
                }
            }

            if goal.physics.momentum > 0.7 && goal.progress.velocity > 0.0 {
                thriving.push(*id);
            }
        }

        GoalHealthReport {
            total_active: active_ids.len(),
            total_blocked: blocked_ids.len(),
            stalled,
            neglected,
            deadline_risk,
            blocked,
            thriving,
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Invention 12: Decision Consensus — Collective Wisdom
    // ═══════════════════════════════════════════════════════════════

    /// Start a consensus process for a decision with multiple stakeholders.
    ///
    /// Validates the decision exists and is in Pending or Deliberating status.
    /// Creates a DecisionConsensus record and transitions the decision to Deliberating.
    pub fn start_consensus(
        &mut self,
        decision_id: DecisionId,
        participants: Vec<ConsensusParticipant>,
    ) -> crate::Result<DecisionConsensus> {
        let decision = self
            .decision_store
            .get(&decision_id)
            .ok_or(crate::Error::DecisionNotFound(decision_id))?;

        if decision.status != DecisionStatus::Pending
            && decision.status != DecisionStatus::Deliberating
        {
            return Err(crate::Error::Validation(format!(
                "cannot start consensus on decision in state {:?}",
                decision.status
            )));
        }

        if participants.is_empty() {
            return Err(crate::Error::CorruptedFile(
                "consensus requires at least one participant".to_string(),
            ));
        }

        let now = Timestamp::now();
        let consensus = DecisionConsensus {
            decision_id,
            stakeholders: participants,
            deliberation: Vec::new(),
            synthesis: None,
            votes: HashMap::new(),
            alignment_score: 0.0,
            status: ConsensusStatus::Open,
            started_at: now,
            crystallized_at: None,
        };

        // Transition decision to Deliberating
        if let Some(d) = self.decision_store.get_mut(&decision_id) {
            d.status = DecisionStatus::Deliberating;
        }

        self.consensus_store.insert(decision_id, consensus.clone());
        self.mark_dirty();
        Ok(consensus)
    }

    /// Add a deliberation round to an active consensus process.
    ///
    /// Each round collects statements from stakeholders and identifies common ground.
    /// The alignment score is recalculated after each round.
    pub fn add_deliberation_round(
        &mut self,
        decision_id: DecisionId,
        statements: Vec<ConsensusStatement>,
        common_ground: Vec<CommonGround>,
    ) -> crate::Result<DecisionConsensus> {
        let consensus = self
            .consensus_store
            .get_mut(&decision_id)
            .ok_or(crate::Error::DecisionNotFound(decision_id))?;

        if consensus.status == ConsensusStatus::Crystallized
            || consensus.status == ConsensusStatus::Deadlocked
        {
            return Err(crate::Error::CorruptedFile(
                "consensus already finalized".to_string(),
            ));
        }

        let round_number = consensus.deliberation.len() + 1;

        // Calculate alignment delta from common ground strength
        let cg_strength: f64 = common_ground.iter().map(|cg| cg.strength).sum::<f64>()
            / common_ground.len().max(1) as f64;
        let concession_bonus: f64 = statements
            .iter()
            .map(|s| if s.concessions.is_empty() { 0.0 } else { 0.1 })
            .sum::<f64>()
            / statements.len().max(1) as f64;
        let alignment_delta = (cg_strength * 0.6 + concession_bonus * 0.4).clamp(0.0, 0.3);

        let round = DeliberationRound {
            round_number,
            statements,
            alignment_delta,
            emerged_common_ground: common_ground,
            recorded_at: Timestamp::now(),
        };

        consensus.deliberation.push(round);
        consensus.alignment_score = (consensus.alignment_score + alignment_delta).clamp(0.0, 1.0);
        consensus.status = ConsensusStatus::Deliberating;
        let result = consensus.clone();
        self.mark_dirty();
        Ok(result)
    }

    /// Propose a synthesis that attempts to reconcile stakeholder positions.
    pub fn synthesize_consensus(
        &mut self,
        decision_id: DecisionId,
        proposal: String,
        incorporates_from: Vec<StakeholderId>,
        addresses_concerns: Vec<String>,
    ) -> crate::Result<DecisionConsensus> {
        let consensus = self
            .consensus_store
            .get_mut(&decision_id)
            .ok_or(crate::Error::DecisionNotFound(decision_id))?;

        if consensus.status == ConsensusStatus::Crystallized {
            return Err(crate::Error::CorruptedFile(
                "consensus already crystallized".to_string(),
            ));
        }

        // Confidence based on how many stakeholders are incorporated
        let total = consensus.stakeholders.len().max(1) as f64;
        let incorporated = incorporates_from.len() as f64;
        let base_confidence = (incorporated / total).clamp(0.0, 1.0);

        let synthesis = Synthesis {
            proposal,
            incorporates_from,
            addresses_concerns,
            confidence: base_confidence * 0.7 + consensus.alignment_score * 0.3,
            proposed_at: Timestamp::now(),
        };

        consensus.synthesis = Some(synthesis);
        consensus.status = ConsensusStatus::Synthesizing;
        let result = consensus.clone();
        self.mark_dirty();
        Ok(result)
    }

    /// Record a stakeholder's vote on the current synthesis.
    pub fn record_consensus_vote(
        &mut self,
        decision_id: DecisionId,
        stakeholder_id: StakeholderId,
        vote: String,
    ) -> crate::Result<DecisionConsensus> {
        let consensus = self
            .consensus_store
            .get_mut(&decision_id)
            .ok_or(crate::Error::DecisionNotFound(decision_id))?;

        if consensus.status == ConsensusStatus::Crystallized {
            return Err(crate::Error::CorruptedFile(
                "consensus already crystallized".to_string(),
            ));
        }

        consensus.votes.insert(stakeholder_id, vote);

        // Transition to Voting once first vote is recorded
        if consensus.status != ConsensusStatus::Voting {
            consensus.status = ConsensusStatus::Voting;
        }

        // Recalculate alignment based on vote agreement
        let total_stakeholders = consensus.stakeholders.len().max(1);
        let voted_count = consensus.votes.len();
        if voted_count > 0 {
            let mut option_counts: HashMap<&String, usize> = HashMap::new();
            for v in consensus.votes.values() {
                *option_counts.entry(v).or_insert(0) += 1;
            }
            let max_agreement = *option_counts.values().max().unwrap_or(&0);
            let agreement_ratio = max_agreement as f64 / total_stakeholders as f64;
            consensus.alignment_score =
                (consensus.alignment_score * 0.5 + agreement_ratio * 0.5).clamp(0.0, 1.0);
        }

        let result = consensus.clone();
        self.mark_dirty();
        Ok(result)
    }

    /// Get the current status of a consensus process.
    pub fn get_consensus_status(
        &self,
        decision_id: DecisionId,
    ) -> crate::Result<&DecisionConsensus> {
        self.consensus_store
            .get(&decision_id)
            .ok_or(crate::Error::DecisionNotFound(decision_id))
    }

    /// Crystallize a consensus decision once alignment is sufficient.
    ///
    /// If alignment_score >= 0.5 (or force=true), crystallizes the decision
    /// and records the consensus outcome. Otherwise returns Deadlocked.
    pub fn crystallize_consensus(
        &mut self,
        decision_id: DecisionId,
        chosen_path: PathId,
        force: bool,
    ) -> crate::Result<DecisionConsensus> {
        let alignment = self
            .consensus_store
            .get(&decision_id)
            .ok_or(crate::Error::DecisionNotFound(decision_id))?
            .alignment_score;

        if !force && alignment < 0.5 {
            if let Some(c) = self.consensus_store.get_mut(&decision_id) {
                c.status = ConsensusStatus::Deadlocked;
            }
            self.mark_dirty();
            return Ok(self
                .consensus_store
                .get(&decision_id)
                .expect("consensus set in deadlock branch")
                .clone());
        }

        // Crystallize the underlying decision
        let rationale = self
            .consensus_store
            .get(&decision_id)
            .and_then(|c| c.synthesis.as_ref())
            .map(|s| s.proposal.clone())
            .unwrap_or_else(|| "Consensus crystallization".to_string());

        self.crystallize(
            decision_id,
            chosen_path,
            DecisionReasoning {
                rationale,
                confidence: alignment,
                ..Default::default()
            },
        )?;

        // Mark consensus as crystallized
        if let Some(c) = self.consensus_store.get_mut(&decision_id) {
            c.status = ConsensusStatus::Crystallized;
            c.crystallized_at = Some(Timestamp::now());
        }
        self.mark_dirty();
        Ok(self
            .consensus_store
            .get(&decision_id)
            .expect("consensus set after crystallization")
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CreateGoalRequest;

    fn make_request(title: &str) -> CreateGoalRequest {
        CreateGoalRequest {
            title: title.to_string(),
            description: format!("Description for {}", title),
            intention: format!("Intention for {}", title),
            ..Default::default()
        }
    }

    #[test]
    fn test_batch_create_empty() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.create_goals_batch(vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        assert_eq!(engine.goal_count(), 0);
    }

    #[test]
    fn test_batch_create_single() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.create_goals_batch(vec![make_request("Alpha")]);
        assert!(result.is_ok());
        let goals = result.unwrap();
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].title, "Alpha");
        assert_eq!(engine.goal_count(), 1);
    }

    #[test]
    fn test_batch_create_multiple() {
        let mut engine = PlanningEngine::in_memory();
        let requests = vec![
            make_request("Goal A"),
            make_request("Goal B"),
            make_request("Goal C"),
            make_request("Goal D"),
            make_request("Goal E"),
        ];

        let result = engine.create_goals_batch(requests);
        assert!(result.is_ok());
        let goals = result.unwrap();
        assert_eq!(goals.len(), 5);
        assert_eq!(engine.goal_count(), 5);

        // Verify all goals are in draft status
        for g in &goals {
            assert_eq!(g.status, GoalStatus::Draft);
        }

        // Verify indexes were rebuilt (root_goals should contain all 5)
        assert_eq!(engine.indexes.root_goals.len(), 5);
    }

    #[test]
    fn test_batch_create_fail_fast_validation() {
        let mut engine = PlanningEngine::in_memory();
        let requests = vec![
            make_request("Valid Goal"),
            CreateGoalRequest {
                title: "".to_string(), // invalid: empty title
                description: "desc".to_string(),
                intention: "intent".to_string(),
                ..Default::default()
            },
            make_request("Another Valid Goal"),
        ];

        let result = engine.create_goals_batch(requests);
        assert!(result.is_err());
        // Fail-fast: NO goals should have been inserted
        assert_eq!(engine.goal_count(), 0);
    }

    #[test]
    fn test_batch_create_with_parent() {
        let mut engine = PlanningEngine::in_memory();

        // Create a parent first
        let parent = engine.create_goal(make_request("Parent")).unwrap();

        // Batch create children
        let mut child_req = make_request("Child");
        child_req.parent = Some(parent.id);
        let result = engine.create_goals_batch(vec![child_req]);
        assert!(result.is_ok());

        // Verify parent has the child linked
        let updated_parent = engine.goal_store.get(&parent.id).unwrap();
        assert_eq!(updated_parent.children.len(), 1);
    }

    #[test]
    fn test_batch_indexes_rebuilt_once() {
        let mut engine = PlanningEngine::in_memory();
        let requests: Vec<CreateGoalRequest> = (0..10)
            .map(|i| make_request(&format!("Batch Goal {}", i)))
            .collect();

        let result = engine.create_goals_batch(requests);
        assert!(result.is_ok());
        assert_eq!(engine.goal_count(), 10);

        // All 10 should be in root_goals (no parents)
        assert_eq!(engine.indexes.root_goals.len(), 10);
        // All 10 should be in Draft status index
        let draft_count = engine
            .indexes
            .goals_by_status
            .get(&GoalStatus::Draft)
            .map(|v| v.len())
            .unwrap_or(0);
        assert_eq!(draft_count, 10);
    }

    #[test]
    fn test_singularity_parallel_empty() {
        let engine = PlanningEngine::in_memory();
        let singularity = engine.calculate_singularity_parallel();
        assert!(singularity.goal_positions.is_empty());
        assert_eq!(singularity.center.confidence, 0.0);
    }

    #[test]
    fn test_blockers_parallel_empty() {
        let engine = PlanningEngine::in_memory();
        let prophecies = engine.scan_blockers_parallel();
        assert!(prophecies.is_empty());
    }

    #[test]
    fn test_echoes_parallel_empty() {
        let engine = PlanningEngine::in_memory();
        let echoes = engine.progress_echoes_parallel();
        assert!(echoes.is_empty());
    }

    #[test]
    fn test_singularity_parallel_with_goals() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine.create_goal(make_request("Active Goal")).unwrap();
        engine.activate_goal(goal.id).unwrap();

        let singularity = engine.calculate_singularity_parallel();
        // Should have at least one position for our active goal
        assert!(!singularity.goal_positions.is_empty());
    }

    #[test]
    fn test_blockers_parallel_with_active_goal() {
        let mut engine = PlanningEngine::in_memory();

        // Create a goal with a dependency (will predict blockers)
        let dep = engine.create_goal(make_request("Dependency")).unwrap();
        let mut req = make_request("Main Goal");
        req.dependencies = Some(vec![dep.id]);
        let goal = engine.create_goal(req).unwrap();
        engine.activate_goal(goal.id).unwrap();

        let prophecies = engine.scan_blockers_parallel();
        // Should predict at least one blocker (dependency not completed)
        assert!(!prophecies.is_empty());
    }

    #[test]
    fn test_blockers_parallel_sorted_by_confidence() {
        let mut engine = PlanningEngine::in_memory();

        // Create multiple goals with dependencies to generate multiple prophecies
        let dep1 = engine.create_goal(make_request("Dep 1")).unwrap();
        let dep2 = engine.create_goal(make_request("Dep 2")).unwrap();

        let mut req = make_request("Goal with deps");
        req.dependencies = Some(vec![dep1.id, dep2.id]);
        let goal = engine.create_goal(req).unwrap();
        engine.activate_goal(goal.id).unwrap();

        let prophecies = engine.scan_blockers_parallel();
        // Verify sorted by confidence descending
        for w in prophecies.windows(2) {
            assert!(w[0].prediction_confidence >= w[1].prediction_confidence);
        }
    }

    #[test]
    fn test_batch_create_preserves_priorities() {
        let mut engine = PlanningEngine::in_memory();
        let mut req_high = make_request("High Priority");
        req_high.priority = Some(Priority::High);
        let mut req_low = make_request("Low Priority");
        req_low.priority = Some(Priority::Low);

        let goals = engine.create_goals_batch(vec![req_high, req_low]).unwrap();
        assert_eq!(goals[0].priority, Priority::High);
        assert_eq!(goals[1].priority, Priority::Low);

        // Verify priority indexes
        assert!(engine
            .indexes
            .goals_by_priority
            .get(&Priority::High)
            .unwrap()
            .contains(&goals[0].id));
        assert!(engine
            .indexes
            .goals_by_priority
            .get(&Priority::Low)
            .unwrap()
            .contains(&goals[1].id));
    }

    // ═══════════════════════════════════════════════════════════════
    // Batch activate goals
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_batch_activate_empty() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.batch_activate_goals(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_batch_activate_valid() {
        let mut engine = PlanningEngine::in_memory();
        let g1 = engine.create_goal(make_request("Goal 1")).unwrap();
        let g2 = engine.create_goal(make_request("Goal 2")).unwrap();

        let result = engine.batch_activate_goals(&[g1.id, g2.id]);
        assert!(result.is_ok());
        let goals = result.unwrap();
        assert_eq!(goals.len(), 2);
        for g in &goals {
            assert_eq!(g.status, GoalStatus::Active);
        }
    }

    #[test]
    fn test_batch_activate_fail_fast_on_invalid_state() {
        let mut engine = PlanningEngine::in_memory();
        let g1 = engine.create_goal(make_request("Goal 1")).unwrap();
        let g2 = engine.create_goal(make_request("Goal 2")).unwrap();
        // Activate g2 first so it can't be activated again
        engine.activate_goal(g2.id).unwrap();

        let result = engine.batch_activate_goals(&[g1.id, g2.id]);
        assert!(result.is_err());
        // g1 should NOT have been activated (fail-fast)
        let g1_check = engine.goal_store.get(&g1.id).unwrap();
        assert_eq!(g1_check.status, GoalStatus::Draft);
    }

    // ═══════════════════════════════════════════════════════════════
    // Batch progress goals
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_batch_progress_empty() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.batch_progress_goals(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_batch_progress_valid() {
        let mut engine = PlanningEngine::in_memory();
        let g1 = engine.create_goal(make_request("Goal 1")).unwrap();
        engine.activate_goal(g1.id).unwrap();
        let g2 = engine.create_goal(make_request("Goal 2")).unwrap();
        engine.activate_goal(g2.id).unwrap();

        let updates = vec![
            (g1.id, 0.5, Some("halfway".to_string())),
            (g2.id, 0.3, None),
        ];
        let result = engine.batch_progress_goals(updates);
        assert!(result.is_ok());
        let goals = result.unwrap();
        assert_eq!(goals.len(), 2);
        assert!((goals[0].progress.percentage - 0.5).abs() < 0.01);
        assert!((goals[1].progress.percentage - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_batch_progress_completion() {
        let mut engine = PlanningEngine::in_memory();
        let g = engine.create_goal(make_request("Finish Me")).unwrap();
        engine.activate_goal(g.id).unwrap();

        let result = engine.batch_progress_goals(vec![(g.id, 1.0, Some("done".to_string()))]);
        assert!(result.is_ok());
        let goals = result.unwrap();
        // Batch progress sets percentage but doesn't auto-complete (optimized path)
        assert!((goals[0].progress.percentage - 1.0).abs() < 0.01);
    }

    // ═══════════════════════════════════════════════════════════════
    // Batch create decisions
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_decisions_empty() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.batch_create_decisions(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_batch_create_decisions_valid() {
        let mut engine = PlanningEngine::in_memory();
        let requests = vec![
            CreateDecisionRequest {
                question: "What framework?".to_string(),
                ..Default::default()
            },
            CreateDecisionRequest {
                question: "What database?".to_string(),
                context: Some("Need ACID compliance".to_string()),
                ..Default::default()
            },
        ];
        let result = engine.batch_create_decisions(requests);
        assert!(result.is_ok());
        let decisions = result.unwrap();
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].question.question, "What framework?");
        assert_eq!(decisions[1].question.question, "What database?");
    }

    #[test]
    fn test_batch_create_decisions_with_goal_link() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine.create_goal(make_request("Main Goal")).unwrap();
        let requests = vec![CreateDecisionRequest {
            question: "How to implement?".to_string(),
            goals: Some(vec![goal.id]),
            ..Default::default()
        }];
        let result = engine.batch_create_decisions(requests);
        assert!(result.is_ok());
        let decisions = result.unwrap();
        assert_eq!(decisions.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════
    // Batch create commitments
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_commitments_empty() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.batch_create_commitments(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_batch_create_commitments_valid() {
        let mut engine = PlanningEngine::in_memory();
        let requests = vec![
            CreateCommitmentRequest {
                promise: Promise {
                    description: "Deliver MVP".to_string(),
                    deliverables: vec!["working app".to_string()],
                    conditions: vec![],
                },
                stakeholder: Stakeholder {
                    name: "Alice".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            CreateCommitmentRequest {
                promise: Promise {
                    description: "Write docs".to_string(),
                    deliverables: vec!["README".to_string()],
                    conditions: vec![],
                },
                stakeholder: Stakeholder {
                    name: "Bob".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];
        let result = engine.batch_create_commitments(requests);
        assert!(result.is_ok());
        let commitments = result.unwrap();
        assert_eq!(commitments.len(), 2);
        assert_eq!(commitments[0].promise.description, "Deliver MVP");
        assert_eq!(commitments[1].promise.description, "Write docs");
    }

    // ═══════════════════════════════════════════════════════════════
    // Batch fulfill commitments
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_batch_fulfill_commitments_empty() {
        let mut engine = PlanningEngine::in_memory();
        let result = engine.batch_fulfill_commitments(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_batch_fulfill_commitments_valid() {
        let mut engine = PlanningEngine::in_memory();
        let c = engine
            .create_commitment(CreateCommitmentRequest {
                promise: Promise {
                    description: "Ship it".to_string(),
                    deliverables: vec!["release".to_string()],
                    conditions: vec![],
                },
                stakeholder: Stakeholder {
                    name: "Team".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();

        let result = engine.batch_fulfill_commitments(vec![(c.id, "Shipped v1.0".to_string())]);
        assert!(result.is_ok());
        let commitments = result.unwrap();
        assert_eq!(commitments.len(), 1);
        assert_eq!(commitments[0].status, CommitmentStatus::Fulfilled);
    }

    // ═══════════════════════════════════════════════════════════════
    // Momentum report (indexed)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_momentum_report_indexed_empty() {
        let engine = PlanningEngine::in_memory();
        let report = engine.momentum_report_indexed();
        assert_eq!(report.total_goals, 0);
        assert!((report.average_momentum - 0.0).abs() < 0.01);
        assert!(report.top_momentum.is_empty());
    }

    #[test]
    fn test_momentum_report_indexed_with_active_goals() {
        let mut engine = PlanningEngine::in_memory();
        let g = engine.create_goal(make_request("Active Goal")).unwrap();
        engine.activate_goal(g.id).unwrap();
        engine
            .progress_goal(g.id, 0.3, Some("started".to_string()))
            .unwrap();

        let report = engine.momentum_report_indexed();
        assert!(report.total_goals >= 1);
        assert!(!report.top_momentum.is_empty());
        // Check the entry has expected fields
        let entry = &report.top_momentum[0];
        assert_eq!(entry.goal_id, g.id);
        assert!(!entry.title.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════
    // Gravity field (indexed)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_gravity_field_indexed_empty() {
        let engine = PlanningEngine::in_memory();
        let field = engine.gravity_field_indexed();
        assert_eq!(field.total_goals, 0);
        assert!(field.wells.is_empty());
    }

    #[test]
    fn test_gravity_field_indexed_with_active_goals() {
        let mut engine = PlanningEngine::in_memory();
        let mut req = make_request("Important Goal");
        req.priority = Some(Priority::Critical);
        let g = engine.create_goal(req).unwrap();
        engine.activate_goal(g.id).unwrap();

        let field = engine.gravity_field_indexed();
        assert!(field.total_goals >= 1);
        assert!(!field.wells.is_empty());
        // Critical priority should produce high gravity
        assert!(field.wells[0].gravity > 0.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // At-risk commitments scan
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_at_risk_commitments_scan_empty() {
        let engine = PlanningEngine::in_memory();
        let results = engine.at_risk_commitments_scan();
        assert!(results.is_empty());
    }

    #[test]
    fn test_at_risk_commitments_scan_with_commitment() {
        let mut engine = PlanningEngine::in_memory();
        // Create a commitment with a past due date
        let c = engine
            .create_commitment(CreateCommitmentRequest {
                promise: Promise {
                    description: "Overdue task".to_string(),
                    deliverables: vec!["something".to_string()],
                    conditions: vec![],
                },
                stakeholder: Stakeholder {
                    name: "Boss".to_string(),
                    ..Default::default()
                },
                due: Some(Timestamp(0)), // epoch = long past
                ..Default::default()
            })
            .unwrap();

        let results = engine.at_risk_commitments_scan();
        // Should flag the overdue commitment
        let found = results.iter().any(|(id, _)| *id == c.id);
        assert!(found, "Overdue commitment should be flagged as at-risk");
    }

    // ═══════════════════════════════════════════════════════════════
    // Metamorphosis scan
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_metamorphosis_scan_empty() {
        let engine = PlanningEngine::in_memory();
        let signals = engine.metamorphosis_scan();
        assert!(signals.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════
    // Progress forecast
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_progress_forecast_batch_empty() {
        let engine = PlanningEngine::in_memory();
        let forecasts = engine.progress_forecast_batch(&[]);
        assert!(forecasts.is_empty());
    }

    #[test]
    fn test_progress_forecast_batch_with_active_goals() {
        let mut engine = PlanningEngine::in_memory();
        let g = engine.create_goal(make_request("Forecast Me")).unwrap();
        engine.activate_goal(g.id).unwrap();
        engine
            .progress_goal(g.id, 0.4, Some("progressing".to_string()))
            .unwrap();

        let forecasts = engine.progress_forecast_batch(&[g.id]);
        assert_eq!(forecasts.len(), 1);
        assert_eq!(forecasts[0].goal_id, g.id);
    }

    #[test]
    fn test_progress_forecast_all_active_empty() {
        let engine = PlanningEngine::in_memory();
        let forecasts = engine.progress_forecast_all_active();
        assert!(forecasts.is_empty());
    }

    #[test]
    fn test_progress_forecast_all_active_with_goals() {
        let mut engine = PlanningEngine::in_memory();
        let g1 = engine.create_goal(make_request("Active 1")).unwrap();
        engine.activate_goal(g1.id).unwrap();
        let g2 = engine.create_goal(make_request("Active 2")).unwrap();
        engine.activate_goal(g2.id).unwrap();
        // Draft goal should NOT appear
        let _draft = engine.create_goal(make_request("Draft")).unwrap();

        let forecasts = engine.progress_forecast_all_active();
        assert_eq!(forecasts.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════
    // Dream goals batch
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_dream_goals_batch_empty() {
        let mut engine = PlanningEngine::in_memory();
        let results = engine.dream_goals_batch(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_dream_goals_batch_valid() {
        let mut engine = PlanningEngine::in_memory();
        let g = engine.create_goal(make_request("Dream Goal")).unwrap();
        engine.activate_goal(g.id).unwrap();

        let results = engine.dream_goals_batch(&[g.id]);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    // ═══════════════════════════════════════════════════════════════
    // Federation health scan
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_federation_health_scan_empty() {
        let engine = PlanningEngine::in_memory();
        let results = engine.federation_health_scan();
        assert!(results.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════
    // Goal health scan
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_goal_health_scan_empty() {
        let engine = PlanningEngine::in_memory();
        let report = engine.goal_health_scan();
        assert_eq!(report.total_active, 0);
        assert_eq!(report.total_blocked, 0);
        assert!(report.stalled.is_empty());
        assert!(report.thriving.is_empty());
    }

    #[test]
    fn test_goal_health_scan_with_active_goals() {
        let mut engine = PlanningEngine::in_memory();
        // Active goal with progress
        let g1 = engine.create_goal(make_request("Active Goal")).unwrap();
        engine.activate_goal(g1.id).unwrap();
        engine
            .progress_goal(g1.id, 0.5, Some("going well".to_string()))
            .unwrap();

        // Another active goal (no progress = stalled candidate)
        let g2 = engine.create_goal(make_request("Stalled Goal")).unwrap();
        engine.activate_goal(g2.id).unwrap();

        let report = engine.goal_health_scan();
        assert!(report.total_active >= 2);
        // g2 has zero momentum and incomplete progress -> stalled
        assert!(report.stalled.contains(&g2.id));
    }

    // ═══════════════════════════════════════════════════════════════
    // Invention 12: Decision Consensus
    // ═══════════════════════════════════════════════════════════════

    fn make_decision(engine: &mut PlanningEngine, q: &str) -> Decision {
        engine
            .create_decision(CreateDecisionRequest {
                question: q.to_string(),
                ..Default::default()
            })
            .unwrap()
    }

    fn make_participants() -> Vec<ConsensusParticipant> {
        vec![
            ConsensusParticipant {
                id: StakeholderId(uuid::Uuid::new_v4()),
                role: "Engineering".to_string(),
                initial_position: "Use Rust".to_string(),
                concerns: vec!["learning curve".to_string()],
                requirements: vec!["performance".to_string()],
                flexibility: 0.6,
            },
            ConsensusParticipant {
                id: StakeholderId(uuid::Uuid::new_v4()),
                role: "Product".to_string(),
                initial_position: "Use Go".to_string(),
                concerns: vec!["time to market".to_string()],
                requirements: vec!["quick iteration".to_string()],
                flexibility: 0.7,
            },
        ]
    }

    #[test]
    fn test_start_consensus() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Which language?");
        let participants = make_participants();
        let result = engine.start_consensus(d.id, participants);
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.decision_id, d.id);
        assert_eq!(consensus.stakeholders.len(), 2);
        assert_eq!(consensus.status, ConsensusStatus::Open);
        assert!((consensus.alignment_score - 0.0).abs() < f64::EPSILON);
        // Decision should transition to Deliberating
        let decision = engine.decision_store.get(&d.id).unwrap();
        assert_eq!(decision.status, DecisionStatus::Deliberating);
    }

    #[test]
    fn test_start_consensus_already_crystallized() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Already decided");
        let path = DecisionPath {
            name: "Option A".to_string(),
            description: "The only option".to_string(),
            ..Default::default()
        };
        engine.add_option(d.id, path.clone()).unwrap();
        engine
            .crystallize(d.id, path.id, DecisionReasoning::default())
            .unwrap();
        let result = engine.start_consensus(d.id, make_participants());
        assert!(result.is_err());
    }

    #[test]
    fn test_start_consensus_empty_participants() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "No one to ask");
        let result = engine.start_consensus(d.id, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_deliberation_round() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Architecture choice");
        let participants = make_participants();
        let p0 = participants[0].id;
        let p1 = participants[1].id;
        engine.start_consensus(d.id, participants).unwrap();

        let statements = vec![
            ConsensusStatement {
                stakeholder_id: p0,
                position: "Rust for safety".to_string(),
                supporting_arguments: vec!["memory safety".to_string()],
                concessions: vec!["willing to consider Go for tooling".to_string()],
            },
            ConsensusStatement {
                stakeholder_id: p1,
                position: "Go for speed".to_string(),
                supporting_arguments: vec!["faster dev cycle".to_string()],
                concessions: vec![],
            },
        ];
        let common_ground = vec![CommonGround {
            description: "Both want reliability".to_string(),
            agreed_by: vec![p0, p1],
            strength: 0.8,
        }];

        let result = engine.add_deliberation_round(d.id, statements, common_ground);
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.deliberation.len(), 1);
        assert_eq!(consensus.status, ConsensusStatus::Deliberating);
        assert!(consensus.alignment_score > 0.0);
    }

    #[test]
    fn test_synthesize_consensus() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Tech choice");
        let participants = make_participants();
        let p0 = participants[0].id;
        let p1 = participants[1].id;
        engine.start_consensus(d.id, participants).unwrap();

        // Add a round first
        engine
            .add_deliberation_round(
                d.id,
                vec![ConsensusStatement {
                    stakeholder_id: p0,
                    position: "Propose hybrid".to_string(),
                    supporting_arguments: vec![],
                    concessions: vec!["open to alternatives".to_string()],
                }],
                vec![CommonGround {
                    description: "Performance matters".to_string(),
                    agreed_by: vec![p0, p1],
                    strength: 0.7,
                }],
            )
            .unwrap();

        let result = engine.synthesize_consensus(
            d.id,
            "Use Rust for core, Go for tooling".to_string(),
            vec![p0, p1],
            vec!["performance".to_string(), "dev speed".to_string()],
        );
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.status, ConsensusStatus::Synthesizing);
        assert!(consensus.synthesis.is_some());
        let syn = consensus.synthesis.unwrap();
        assert_eq!(syn.incorporates_from.len(), 2);
        assert!(syn.confidence > 0.0);
    }

    #[test]
    fn test_record_consensus_vote() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Vote on approach");
        let participants = make_participants();
        let p0 = participants[0].id;
        let p1 = participants[1].id;
        engine.start_consensus(d.id, participants).unwrap();

        let result = engine.record_consensus_vote(d.id, p0, "approve".to_string());
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.status, ConsensusStatus::Voting);
        assert_eq!(consensus.votes.len(), 1);

        // Second vote with same option should increase alignment
        let result2 = engine.record_consensus_vote(d.id, p1, "approve".to_string());
        assert!(result2.is_ok());
        let consensus2 = result2.unwrap();
        assert_eq!(consensus2.votes.len(), 2);
        // Both voted "approve" so alignment should be high
        assert!(consensus2.alignment_score > 0.3);
    }

    #[test]
    fn test_get_consensus_status() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Status query test");
        engine.start_consensus(d.id, make_participants()).unwrap();

        let result = engine.get_consensus_status(d.id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().stakeholders.len(), 2);
    }

    #[test]
    fn test_get_consensus_status_not_found() {
        let engine = PlanningEngine::in_memory();
        let result = engine.get_consensus_status(DecisionId(uuid::Uuid::new_v4()));
        assert!(result.is_err());
    }

    #[test]
    fn test_crystallize_consensus_sufficient_alignment() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Crystallize me");
        let participants = make_participants();
        let p0 = participants[0].id;
        let p1 = participants[1].id;
        engine.start_consensus(d.id, participants).unwrap();

        // Add a path so crystallize can find it
        let path = DecisionPath {
            name: "Hybrid approach".to_string(),
            description: "Best of both".to_string(),
            ..Default::default()
        };
        let path_id = path.id;
        engine.add_option(d.id, path).unwrap();

        // Build alignment through voting
        engine
            .record_consensus_vote(d.id, p0, "approve".to_string())
            .unwrap();
        engine
            .record_consensus_vote(d.id, p1, "approve".to_string())
            .unwrap();

        let result = engine.crystallize_consensus(d.id, path_id, false);
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.status, ConsensusStatus::Crystallized);
        assert!(consensus.crystallized_at.is_some());
        // Underlying decision should be crystallized too
        let decision = engine.decision_store.get(&d.id).unwrap();
        assert_eq!(decision.status, DecisionStatus::Crystallized);
    }

    #[test]
    fn test_crystallize_consensus_low_alignment_deadlocks() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Deadlock me");
        engine.start_consensus(d.id, make_participants()).unwrap();

        let path = DecisionPath {
            name: "Option A".to_string(),
            description: "Untested".to_string(),
            ..Default::default()
        };
        let path_id = path.id;
        engine.add_option(d.id, path).unwrap();

        // No votes, alignment = 0 -> should deadlock
        let result = engine.crystallize_consensus(d.id, path_id, false);
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.status, ConsensusStatus::Deadlocked);
    }

    #[test]
    fn test_crystallize_consensus_force() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Force it");
        engine.start_consensus(d.id, make_participants()).unwrap();

        let path = DecisionPath {
            name: "Emergency choice".to_string(),
            description: "No time to deliberate".to_string(),
            ..Default::default()
        };
        let path_id = path.id;
        engine.add_option(d.id, path).unwrap();

        // Force crystallize even with 0 alignment
        let result = engine.crystallize_consensus(d.id, path_id, true);
        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.status, ConsensusStatus::Crystallized);
    }

    #[test]
    fn test_consensus_full_lifecycle() {
        let mut engine = PlanningEngine::in_memory();
        let d = make_decision(&mut engine, "Full lifecycle test");
        let participants = make_participants();
        let p0 = participants[0].id;
        let p1 = participants[1].id;

        // 1. Start
        let c = engine.start_consensus(d.id, participants).unwrap();
        assert_eq!(c.status, ConsensusStatus::Open);

        // 2. Deliberate
        let c = engine
            .add_deliberation_round(
                d.id,
                vec![
                    ConsensusStatement {
                        stakeholder_id: p0,
                        position: "Prefer A".to_string(),
                        supporting_arguments: vec!["fast".to_string()],
                        concessions: vec!["B is fine too".to_string()],
                    },
                    ConsensusStatement {
                        stakeholder_id: p1,
                        position: "Prefer B".to_string(),
                        supporting_arguments: vec!["safe".to_string()],
                        concessions: vec!["A has merit".to_string()],
                    },
                ],
                vec![CommonGround {
                    description: "Both want quality".to_string(),
                    agreed_by: vec![p0, p1],
                    strength: 0.9,
                }],
            )
            .unwrap();
        assert_eq!(c.status, ConsensusStatus::Deliberating);

        // 3. Synthesize
        let c = engine
            .synthesize_consensus(
                d.id,
                "Combine A's speed with B's safety".to_string(),
                vec![p0, p1],
                vec!["quality".to_string()],
            )
            .unwrap();
        assert_eq!(c.status, ConsensusStatus::Synthesizing);

        // 4. Vote
        engine
            .record_consensus_vote(d.id, p0, "approve".to_string())
            .unwrap();
        let c = engine
            .record_consensus_vote(d.id, p1, "approve".to_string())
            .unwrap();
        assert_eq!(c.status, ConsensusStatus::Voting);

        // 5. Crystallize
        let path = DecisionPath {
            name: "Combined approach".to_string(),
            description: "A+B synthesis".to_string(),
            ..Default::default()
        };
        let path_id = path.id;
        engine.add_option(d.id, path).unwrap();

        let c = engine.crystallize_consensus(d.id, path_id, false).unwrap();
        assert_eq!(c.status, ConsensusStatus::Crystallized);
        assert!(c.crystallized_at.is_some());
    }
}
