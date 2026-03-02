use crate::error::{Error, Result};
use crate::types::*;
use crate::PlanningEngine;
use std::collections::HashMap;

impl PlanningEngine {
    pub fn get_goal(&self, id: GoalId) -> Option<&Goal> {
        self.goal_store.get(&id)
    }

    pub fn list_goals(&self, filter: GoalFilter) -> Vec<&Goal> {
        let mut goals: Vec<&Goal> = self
            .goal_store
            .values()
            .filter(|g| self.matches_goal_filter(g, &filter))
            .collect();

        goals.sort_by_key(|g| g.created_at);

        if let Some(limit) = filter.limit {
            goals.truncate(limit);
        }

        goals
    }

    pub fn get_root_goals(&self) -> Vec<&Goal> {
        self.indexes
            .root_goals
            .iter()
            .filter_map(|id| self.goal_store.get(id))
            .collect()
    }

    pub fn get_active_goals(&self) -> Vec<&Goal> {
        self.indexes
            .active_goals
            .iter()
            .filter_map(|id| self.goal_store.get(id))
            .collect()
    }

    pub fn get_blocked_goals(&self) -> Vec<&Goal> {
        self.indexes
            .blocked_goals
            .iter()
            .filter_map(|id| self.goal_store.get(id))
            .collect()
    }

    pub fn get_urgent_goals(&self, within_days: f64) -> Vec<&Goal> {
        let now = Timestamp::now();
        let cutoff = Timestamp(now.0 + (within_days * 86_400.0 * 1e9) as i64);

        self.indexes
            .goals_by_deadline
            .iter()
            .filter(|(deadline, _)| **deadline <= cutoff)
            .flat_map(|(_, ids)| ids.iter())
            .filter_map(|id| self.goal_store.get(id))
            .filter(|g| g.status == GoalStatus::Active || g.status == GoalStatus::Blocked)
            .collect()
    }

    pub fn get_goal_tree(&self, root_id: GoalId) -> Option<GoalTree> {
        self.goal_store.get(&root_id)?;
        let mut tree = GoalTree {
            root: root_id,
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        self.build_tree_recursive(root_id, 0, &mut tree);
        Some(tree)
    }

    fn build_tree_recursive(&self, id: GoalId, depth: usize, tree: &mut GoalTree) {
        if let Some(goal) = self.goal_store.get(&id) {
            tree.nodes.insert(
                id,
                GoalTreeNode {
                    goal: goal.clone(),
                    depth,
                },
            );
            for child_id in &goal.children {
                tree.edges.push((id, *child_id));
                self.build_tree_recursive(*child_id, depth + 1, tree);
            }
        }
    }

    pub fn search_goals(&self, query: &str) -> Vec<&Goal> {
        let q = query.to_lowercase();
        self.goal_store
            .values()
            .filter(|g| {
                g.title.to_lowercase().contains(&q)
                    || g.description.to_lowercase().contains(&q)
                    || g.soul.intention.to_lowercase().contains(&q)
                    || g.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn get_intention_singularity(&self) -> IntentionSingularity {
        let active_goals: Vec<_> = self.get_active_goals().into_iter().cloned().collect();

        if active_goals.is_empty() {
            return IntentionSingularity::default();
        }

        let center = self.calculate_intention_center(&active_goals);

        let positions = active_goals
            .iter()
            .map(|g| {
                (
                    g.id,
                    IntentionPosition {
                        goal_id: g.id,
                        centrality: self.calculate_centrality(g, &center),
                        alignment_angle: self.calculate_alignment(g, &center),
                        gravitational_pull: g.physics.gravity,
                        drift_risk: g.feelings.neglect,
                    },
                )
            })
            .collect();

        IntentionSingularity {
            unified_vision: self.synthesize_vision(&active_goals),
            goal_positions: positions,
            themes: self.extract_themes(&active_goals),
            tension_lines: self.find_tensions(&active_goals),
            golden_path: self.calculate_optimal_path(&active_goals),
            center,
        }
    }

    pub fn get_decision(&self, id: DecisionId) -> Option<&Decision> {
        self.decision_store.get(&id)
    }

    pub fn get_decision_chain(&self, id: DecisionId) -> Option<DecisionChain> {
        self.decision_store.get(&id)?;

        let mut root = id;
        while let Some(parent_id) = self.decision_store.get(&root).and_then(|d| d.caused_by) {
            root = parent_id;
        }

        let mut chain = DecisionChain {
            root,
            descendants: Vec::new(),
            causality: Vec::new(),
            cascade_analysis: CascadeAnalysis::default(),
        };

        self.build_chain_recursive(root, &mut chain);
        chain.cascade_analysis.total_nodes = chain.descendants.len() + 1;
        Some(chain)
    }

    fn build_chain_recursive(&self, id: DecisionId, chain: &mut DecisionChain) {
        if let Some(decision) = self.decision_store.get(&id) {
            for child_id in &decision.causes {
                if let Some(child) = self.decision_store.get(child_id) {
                    // Determine causality type from relationship characteristics
                    let causality_type = self.infer_causality_type(decision, child);
                    let strength = self.infer_causality_strength(decision, child);

                    chain.descendants.push(*child_id);
                    chain.causality.push(CausalLink {
                        from: id,
                        to: *child_id,
                        causality_type,
                        strength,
                    });
                } else {
                    // Child not found — still record with default
                    chain.descendants.push(*child_id);
                    chain.causality.push(CausalLink {
                        from: id,
                        to: *child_id,
                        causality_type: CausalityType::Enables,
                        strength: 0.5,
                    });
                }
                self.build_chain_recursive(*child_id, chain);
            }
        }
    }

    fn infer_causality_type(&self, parent: &Decision, child: &Decision) -> CausalityType {
        // If parent and child affect overlapping goals, they have a stronger causal link
        let shared_goals = parent
            .affected_goals
            .iter()
            .filter(|g| child.affected_goals.contains(g))
            .count();

        // Check if child's consequences constrain parent's path
        let has_negative_consequences = child
            .consequences
            .iter()
            .any(|c| matches!(c.impact, Impact::Negative));

        // If the parent was caused_by the child, it's a direct requirement
        if parent.caused_by == Some(child.id) {
            return CausalityType::Requires;
        }

        // If child has negative consequences that overlap with parent's goals, it constrains
        if has_negative_consequences && shared_goals > 0 {
            return CausalityType::Constrains;
        }

        // If child is crystallized and parent isn't, child suggests the parent's direction
        if child.status == DecisionStatus::Crystallized
            && parent.status != DecisionStatus::Crystallized
        {
            return CausalityType::Suggests;
        }

        // Strong shared goal overlap implies direct enablement
        if shared_goals >= 2 {
            return CausalityType::Requires;
        }

        CausalityType::Enables
    }

    fn infer_causality_strength(&self, parent: &Decision, child: &Decision) -> f64 {
        let mut strength = 0.5;

        // Shared affected goals increase strength
        let shared = parent
            .affected_goals
            .iter()
            .filter(|g| child.affected_goals.contains(g))
            .count();
        strength += shared as f64 * 0.15;

        // Crystallized decisions have stronger links
        if child.status == DecisionStatus::Crystallized {
            strength += 0.2;
        }

        // High confidence reasoning increases link strength
        strength += child.reasoning.confidence * 0.1;

        strength.clamp(0.1, 1.0)
    }

    pub fn get_shadows(&self, id: DecisionId) -> Vec<&CrystalShadow> {
        self.decision_store
            .get(&id)
            .map(|d| d.shadows.iter().collect())
            .unwrap_or_default()
    }

    pub fn project_counterfactual(
        &self,
        decision_id: DecisionId,
        path_id: PathId,
    ) -> Option<CounterfactualProjection> {
        let decision = self.decision_store.get(&decision_id)?;
        let shadow = decision.shadows.iter().find(|s| s.path.id == path_id)?;

        Some(CounterfactualProjection {
            projected_at: Timestamp::now(),
            timeline: self.generate_projected_timeline(decision, &shadow.path),
            final_state: self.project_final_state(decision, &shadow.path),
            confidence: self.calculate_projection_confidence(decision),
        })
    }

    pub fn decision_archaeology(&self, artifact: &str) -> DecisionArchaeology {
        let mut relevant: Vec<_> = self
            .decision_store
            .values()
            .filter(|d| {
                d.question.question.contains(artifact) || d.question.context.contains(artifact)
            })
            .collect();

        relevant.sort_by_key(|d| d.crystallized_at.unwrap_or(Timestamp(0)));

        let strata: Vec<ArchaeologicalStratum> = relevant
            .iter()
            .enumerate()
            .map(|(i, d)| {
                // Assess reasonableness from consequence ratio
                let positive = d
                    .consequences
                    .iter()
                    .filter(|c| matches!(c.impact, Impact::Positive))
                    .count();
                let negative = d
                    .consequences
                    .iter()
                    .filter(|c| matches!(c.impact, Impact::Negative))
                    .count();
                let total = d.consequences.len().max(1);
                let was_reasonable = if d.consequences.is_empty() {
                    // No consequences recorded yet — give benefit of the doubt
                    d.reasoning.confidence >= 0.4
                } else {
                    positive >= negative
                };

                let modern_assessment = if d.consequences.is_empty() {
                    format!(
                        "Pending assessment (confidence: {:.0}%)",
                        d.reasoning.confidence * 100.0
                    )
                } else if was_reasonable {
                    format!("Reasonable: {}/{} positive outcomes", positive, total)
                } else {
                    format!("Questionable: {}/{} negative outcomes", negative, total)
                };

                ArchaeologicalStratum {
                    depth: relevant.len() - i,
                    decision: d.id,
                    age: self.calculate_age(d),
                    impact_on_artifact: format!("{:?}", d.chosen.as_ref().map(|c| &c.name)),
                    context_at_time: d.question.context.clone(),
                    was_reasonable,
                    modern_assessment,
                }
            })
            .collect();

        // Generate insights from patterns across the strata
        let mut insights = Vec::new();
        let unreasonable_count = strata.iter().filter(|s| !s.was_reasonable).count();
        if unreasonable_count > 0 {
            insights.push(format!(
                "{} of {} decisions about '{}' had questionable outcomes",
                unreasonable_count,
                strata.len(),
                artifact
            ));
        }
        if strata.len() >= 3 {
            insights.push(format!(
                "'{}' has been a recurring decision point ({} times) — consider a standing policy",
                artifact,
                strata.len()
            ));
        }
        if relevant.iter().any(|d| {
            d.status == DecisionStatus::Regretted || d.status == DecisionStatus::Recrystallized
        }) {
            insights.push(format!(
                "At least one decision about '{}' was regretted or recrystallized — pattern may be unstable",
                artifact
            ));
        }

        DecisionArchaeology {
            artifact: artifact.to_string(),
            strata,
            cumulative_impact: self.calculate_cumulative_impact(&relevant),
            insights,
        }
    }

    pub fn get_commitment(&self, id: CommitmentId) -> Option<&Commitment> {
        self.commitment_store.get(&id)
    }

    pub fn get_dream(&self, id: DreamId) -> Option<&Dream> {
        self.dream_store.get(&id)
    }

    pub fn list_dreams(&self) -> Vec<&Dream> {
        self.dream_store.values().collect()
    }

    pub fn list_goal_dreams(&self, goal_id: GoalId) -> Vec<&Dream> {
        self.dream_store
            .values()
            .filter(|d| d.goal_id == goal_id)
            .collect()
    }

    pub fn list_decisions(&self) -> Vec<&Decision> {
        self.decision_store.values().collect()
    }

    pub fn list_commitments(&self) -> Vec<&Commitment> {
        self.commitment_store.values().collect()
    }

    pub fn get_due_soon(&self, within_days: f64) -> Vec<&Commitment> {
        let now = Timestamp::now();
        let cutoff = Timestamp(now.0 + (within_days * 86_400.0 * 1e9) as i64);

        self.indexes
            .commitments_by_due
            .iter()
            .filter(|(due, _)| **due <= cutoff)
            .flat_map(|(_, ids)| ids.iter())
            .filter_map(|id| self.commitment_store.get(id))
            .filter(|c| c.status == CommitmentStatus::Active)
            .collect()
    }

    pub fn get_commitment_inventory(&self) -> CommitmentInventory {
        let commitments: Vec<_> = self.commitment_store.values().collect();
        let total_weight: f64 = commitments
            .iter()
            .filter(|c| c.status == CommitmentStatus::Active)
            .map(|c| c.weight)
            .sum();

        CommitmentInventory {
            total_count: commitments.len(),
            active_count: commitments
                .iter()
                .filter(|c| c.status == CommitmentStatus::Active)
                .count(),
            total_weight,
            sustainable_weight: 2.0,
            is_overloaded: total_weight > 2.0,
            by_stakeholder: self.group_by_stakeholder(&commitments),
        }
    }

    pub fn get_at_risk_commitments(&self) -> Vec<&Commitment> {
        self.commitment_store
            .values()
            .filter(|c| c.status == CommitmentStatus::Active && self.is_at_risk(c))
            .collect()
    }

    pub fn get_federation(&self, id: FederationId) -> Option<&Federation> {
        self.federation_store.get(&id)
    }

    pub fn list_federations(&self) -> Vec<&Federation> {
        self.federation_store.values().collect()
    }

    pub fn get_federation_members(&self, id: FederationId) -> Option<Vec<FederationMember>> {
        self.federation_store.get(&id).map(|f| f.members.clone())
    }

    pub fn scan_blocker_prophecy(&self) -> Vec<BlockerProphecy> {
        let mut prophecies = Vec::new();

        // Count blocker type frequency across all goals for confidence calibration
        let mut blocker_type_counts: HashMap<String, usize> = HashMap::new();
        for goal in self.goal_store.values() {
            for blocker in &goal.blockers {
                let key = format!("{:?}", std::mem::discriminant(&blocker.blocker_type));
                *blocker_type_counts.entry(key).or_insert(0) += 1;
            }
        }
        let total_historical = blocker_type_counts.values().sum::<usize>().max(1);

        for goal in self.get_active_goals() {
            for blocker in self.predict_blockers(goal) {
                // Confidence from: blocker type recurrence + goal severity signals
                let type_key = format!("{:?}", std::mem::discriminant(&blocker.blocker_type));
                let type_frequency = *blocker_type_counts.get(&type_key).unwrap_or(&0) as f64
                    / total_historical as f64;
                let severity_signal = blocker.severity;
                let prediction_confidence =
                    (0.3 + type_frequency * 0.3 + severity_signal * 0.3).clamp(0.1, 0.95);

                // Days until materialization from deadline proximity and severity
                let days_until = goal
                    .deadline
                    .map(|d| {
                        let days = ((d.0 - Timestamp::now().0) as f64 / (86_400.0 * 1e9)).max(0.5);
                        (days * (1.0 - severity_signal)).max(1.0)
                    })
                    .unwrap_or(14.0 * (1.0 - severity_signal * 0.5));

                // Build evidence from goal state
                let mut evidence = Vec::new();
                if goal.progress.velocity == 0.0 {
                    evidence.push("zero progress velocity".to_string());
                }
                if goal.feelings.neglect > 0.5 {
                    evidence.push(format!("high neglect score ({:.2})", goal.feelings.neglect));
                }
                if !goal.dependencies.is_empty() {
                    let incomplete_deps = goal
                        .dependencies
                        .iter()
                        .filter(|d| {
                            self.goal_store
                                .get(d)
                                .map(|g| g.status != GoalStatus::Completed)
                                .unwrap_or(true)
                        })
                        .count();
                    if incomplete_deps > 0 {
                        evidence.push(format!("{} incomplete dependencies", incomplete_deps));
                    }
                }

                // Recommended actions based on blocker type
                let recommended_actions = match &blocker.blocker_type {
                    BlockerType::DependencyBlocked { goal: dep_id } => {
                        let dep_name = self
                            .goal_store
                            .get(dep_id)
                            .map(|g| g.title.clone())
                            .unwrap_or_else(|| format!("{:?}", dep_id));
                        vec![format!("Prioritize completing '{}'", dep_name)]
                    }
                    BlockerType::ResourceUnavailable { resource } => {
                        vec![format!("Secure resource: {}", resource)]
                    }
                    BlockerType::ExternalEvent { event } => {
                        vec![format!("Monitor and prepare for: {}", event)]
                    }
                    BlockerType::SkillGap { skill } => {
                        vec![format!("Acquire skill or delegate: {}", skill)]
                    }
                    BlockerType::ApprovalPending { .. } => {
                        vec!["Follow up on approval request".to_string()]
                    }
                    BlockerType::TechnicalDebt { description } => {
                        vec![format!("Address technical debt: {}", description)]
                    }
                    BlockerType::DeadlineMiss { .. } => {
                        vec!["Renegotiate deadline or increase resources".to_string()]
                    }
                    BlockerType::Unknown { signals } => {
                        if signals.is_empty() {
                            vec!["Investigate root cause".to_string()]
                        } else {
                            vec![format!("Investigate signals: {}", signals.join(", "))]
                        }
                    }
                };

                prophecies.push(BlockerProphecy {
                    goal_id: goal.id,
                    predicted_blocker: blocker,
                    prediction_confidence,
                    days_until_materialization: days_until,
                    evidence,
                    recommended_actions,
                });
            }
        }
        prophecies
    }

    pub fn listen_progress_echoes(&self) -> Vec<ProgressEcho> {
        let mut echoes = Vec::new();
        for goal in self.get_active_goals() {
            if goal.progress.percentage > 0.5 && goal.physics.momentum > 0.2 {
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
        }
        echoes
    }

    pub fn get_decision_prophecy(
        &self,
        question: &str,
        options: &[DecisionPath],
    ) -> DecisionProphecy {
        let paths = options
            .iter()
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

    fn matches_goal_filter(&self, goal: &Goal, filter: &GoalFilter) -> bool {
        if let Some(statuses) = &filter.status {
            if !statuses.contains(&goal.status) {
                return false;
            }
        }
        if let Some(priorities) = &filter.priority {
            if !priorities.contains(&goal.priority) {
                return false;
            }
        }
        if let Some(parent) = filter.parent {
            if goal.parent != Some(parent) {
                return false;
            }
        }
        if let Some(has_deadline) = filter.has_deadline {
            if goal.deadline.is_some() != has_deadline {
                return false;
            }
        }
        if let Some(before) = filter.deadline_before {
            if goal.deadline.map(|d| d > before).unwrap_or(true) {
                return false;
            }
        }
        if let Some(after) = filter.deadline_after {
            if goal.deadline.map(|d| d < after).unwrap_or(true) {
                return false;
            }
        }
        if let Some(tags) = &filter.tags {
            if !tags.iter().all(|t| goal.tags.contains(t)) {
                return false;
            }
        }
        if let Some(created_after) = filter.created_after {
            if goal.created_at < created_after {
                return false;
            }
        }
        if let Some(min) = filter.min_progress {
            if goal.progress.percentage < min {
                return false;
            }
        }
        if let Some(max) = filter.max_progress {
            if goal.progress.percentage > max {
                return false;
            }
        }
        if let Some(min_momentum) = filter.min_momentum {
            if goal.physics.momentum < min_momentum {
                return false;
            }
        }
        true
    }

    fn calculate_intention_center(&self, goals: &[Goal]) -> IntentionCenter {
        if goals.is_empty() {
            return IntentionCenter {
                urgency: 0.0,
                confidence: 0.0,
                momentum: 0.0,
            };
        }

        let len = goals.len() as f64;
        IntentionCenter {
            urgency: goals.iter().map(|g| g.feelings.urgency).sum::<f64>() / len,
            confidence: goals.iter().map(|g| g.feelings.confidence).sum::<f64>() / len,
            momentum: goals.iter().map(|g| g.physics.momentum).sum::<f64>() / len,
        }
    }

    fn calculate_centrality(&self, goal: &Goal, center: &IntentionCenter) -> f64 {
        let d = (goal.feelings.urgency - center.urgency).abs()
            + (goal.feelings.confidence - center.confidence).abs()
            + (goal.physics.momentum - center.momentum).abs();
        (1.0 - (d / 3.0)).clamp(0.0, 1.0)
    }

    fn calculate_alignment(&self, goal: &Goal, center: &IntentionCenter) -> f64 {
        ((goal.physics.momentum + goal.feelings.confidence + goal.feelings.urgency)
            - (center.momentum + center.confidence + center.urgency))
            .atan()
    }

    fn synthesize_vision(&self, goals: &[Goal]) -> String {
        if goals.is_empty() {
            return "No active intentions".to_string();
        }

        let mut top = goals.to_vec();
        top.sort_by(|a, b| {
            let sa = a.physics.gravity + a.feelings.urgency;
            let sb = b.physics.gravity + b.feelings.urgency;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Group by parent to detect clusters
        let mut root_clusters: HashMap<Option<GoalId>, Vec<&Goal>> = HashMap::new();
        for g in &top {
            root_clusters.entry(g.parent).or_default().push(g);
        }

        // Identify dominant themes from tags
        let themes = self.extract_themes(goals);
        let theme_str = if themes.is_empty() {
            String::new()
        } else {
            format!(
                " Themes: {}.",
                themes
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        // Detect conflicts (goals with opposing momentum)
        let stalled: Vec<_> = top
            .iter()
            .filter(|g| g.progress.velocity == 0.0 && g.feelings.urgency > 0.6)
            .collect();
        let conflict_str = if !stalled.is_empty() {
            format!(" Warning: {} urgent goal(s) stalled.", stalled.len())
        } else {
            String::new()
        };

        // Build vision from top goals with cluster context
        let primary: Vec<String> = top.iter().take(3).map(|g| g.title.clone()).collect();
        let cluster_count = root_clusters.len();

        if cluster_count == 1 {
            format!(
                "Unified focus: {}.{}{}",
                primary.join(", "),
                theme_str,
                conflict_str
            )
        } else {
            format!(
                "Focus across {} streams: {}.{}{}",
                cluster_count,
                primary.join(", "),
                theme_str,
                conflict_str
            )
        }
    }

    fn extract_themes(&self, goals: &[Goal]) -> Vec<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for g in goals {
            for t in &g.tags {
                *counts.entry(t.to_lowercase()).or_insert(0) += 1;
            }
        }
        let mut pairs: Vec<_> = counts.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.into_iter().take(5).map(|(k, _)| k).collect()
    }

    fn find_tensions(&self, goals: &[Goal]) -> Vec<TensionLine> {
        let mut tensions = Vec::new();
        for i in 0..goals.len() {
            for j in (i + 1)..goals.len() {
                let a = &goals[i];
                let b = &goals[j];

                // 1. Urgency divergence
                let urgency_delta = (a.feelings.urgency - b.feelings.urgency).abs();
                if urgency_delta > 0.4 {
                    tensions.push(TensionLine {
                        a: a.id,
                        b: b.id,
                        magnitude: urgency_delta,
                        reason: "urgency divergence".to_string(),
                    });
                }

                // 2. Resource conflict: shared stakeholders via commitments
                let a_stakeholders: Vec<_> = a
                    .commitments
                    .iter()
                    .filter_map(|cid| self.commitment_store.get(cid))
                    .map(|c| c.made_to.id)
                    .collect();
                let b_stakeholders: Vec<_> = b
                    .commitments
                    .iter()
                    .filter_map(|cid| self.commitment_store.get(cid))
                    .map(|c| c.made_to.id)
                    .collect();
                let shared_stakeholders = a_stakeholders
                    .iter()
                    .filter(|s| b_stakeholders.contains(s))
                    .count();
                if shared_stakeholders > 0 {
                    tensions.push(TensionLine {
                        a: a.id,
                        b: b.id,
                        magnitude: (shared_stakeholders as f64 * 0.3).min(1.0),
                        reason: format!(
                            "shared stakeholder conflict ({} shared)",
                            shared_stakeholders
                        ),
                    });
                }

                // 3. Timeline conflict: overlapping deadlines with dependency relationship
                if let (Some(da), Some(db)) = (a.deadline, b.deadline) {
                    let days_apart = ((da.0 - db.0) as f64 / (86_400.0 * 1e9)).abs();
                    if days_apart < 3.0
                        && (a.dependencies.contains(&b.id) || b.dependencies.contains(&a.id))
                    {
                        tensions.push(TensionLine {
                            a: a.id,
                            b: b.id,
                            magnitude: (1.0 - days_apart / 3.0).clamp(0.3, 1.0),
                            reason: "deadline collision with dependency".to_string(),
                        });
                    }
                }

                // 4. Energy conflict: both high-gravity goals competing for momentum
                if a.physics.gravity > 0.7 && b.physics.gravity > 0.7 {
                    let combined_energy_demand = a.physics.gravity + b.physics.gravity;
                    if combined_energy_demand > 1.5 {
                        tensions.push(TensionLine {
                            a: a.id,
                            b: b.id,
                            magnitude: (combined_energy_demand - 1.5).min(1.0),
                            reason: "energy competition (both high gravity)".to_string(),
                        });
                    }
                }
            }
        }
        tensions
    }

    fn calculate_optimal_path(&self, goals: &[Goal]) -> Vec<GoalId> {
        let mut ranked = goals.to_vec();
        ranked.sort_by(|a, b| {
            let sa = a.physics.gravity + a.feelings.urgency + a.physics.momentum;
            let sb = b.physics.gravity + b.feelings.urgency + b.physics.momentum;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked.into_iter().map(|g| g.id).collect()
    }

    fn calculate_age(&self, decision: &Decision) -> String {
        let at = decision
            .crystallized_at
            .unwrap_or(decision.question.asked_at);
        let days = ((Timestamp::now().0 - at.0) as f64 / (86_400.0 * 1e9)).max(0.0);
        format!("{days:.1} days")
    }

    fn calculate_cumulative_impact(&self, decisions: &[&Decision]) -> String {
        let crystallized = decisions
            .iter()
            .filter(|d| {
                d.status == DecisionStatus::Crystallized
                    || d.status == DecisionStatus::Recrystallized
            })
            .count();
        format!("{} crystallized layers", crystallized)
    }

    fn group_by_stakeholder(&self, commitments: &[&Commitment]) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for c in commitments {
            *map.entry(c.made_to.name.clone()).or_insert(0) += 1;
        }
        map
    }

    fn is_at_risk(&self, commitment: &Commitment) -> bool {
        let Some(due) = commitment.due else {
            return false;
        };

        let now = Timestamp::now();
        let days_remaining = (due.0 - now.0) as f64 / (86_400.0 * 1e9);

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

        days_remaining < 7.0
    }

    pub(crate) fn predict_blockers(&self, goal: &Goal) -> Vec<Blocker> {
        let mut blockers = Vec::new();

        if goal.progress.velocity == 0.0 && goal.progress.percentage < 0.2 {
            blockers.push(Blocker {
                id: uuid::Uuid::new_v4(),
                blocker_type: BlockerType::Unknown {
                    signals: vec!["no progress velocity".to_string()],
                },
                description: "Execution stall likely".to_string(),
                severity: 0.6,
                identified_at: Timestamp::now(),
                resolved_at: None,
                resolution: None,
            });
        }

        if goal.feelings.neglect > 0.6 {
            blockers.push(Blocker {
                id: uuid::Uuid::new_v4(),
                blocker_type: BlockerType::ExternalEvent {
                    event: "attention drift".to_string(),
                },
                description: "Attention drift may block completion".to_string(),
                severity: 0.5,
                identified_at: Timestamp::now(),
                resolved_at: None,
                resolution: None,
            });
        }

        blockers
    }

    pub(crate) fn extract_echo_info(&self, goal: &Goal) -> Vec<String> {
        vec![
            format!("momentum:{:.2}", goal.physics.momentum),
            format!("velocity:{:.3}", goal.progress.velocity),
            format!("confidence:{:.2}", goal.feelings.confidence),
        ]
    }

    fn generate_projected_timeline(
        &self,
        decision: &Decision,
        path: &DecisionPath,
    ) -> Vec<ProjectedEvent> {
        let mut events = Vec::new();
        let risk = path.estimated_risk.unwrap_or(0.5);
        let effort = path.estimated_effort.unwrap_or(0.5);

        // Compute aggregate momentum and velocity from affected goals
        let (avg_momentum, avg_velocity) = if decision.affected_goals.is_empty() {
            (0.5, 0.3)
        } else {
            let mut total_momentum = 0.0;
            let mut total_velocity = 0.0;
            let mut count = 0.0;
            for gid in &decision.affected_goals {
                if let Some(goal) = self.goal_store.get(gid) {
                    total_momentum += goal.physics.momentum;
                    total_velocity += goal.progress.velocity;
                    count += 1.0;
                }
            }
            if count > 0.0 {
                (total_momentum / count, total_velocity / count)
            } else {
                (0.5, 0.3)
            }
        };

        // Phase 1: Adoption/initiation — scaled by effort and momentum
        let adoption_days = (7.0 * effort / avg_momentum.max(0.1)).clamp(2.0, 30.0);
        let adoption_prob = (0.6 + avg_momentum * 0.3 - risk * 0.2).clamp(0.2, 0.95);
        events.push(ProjectedEvent {
            time_offset_days: adoption_days,
            event: format!("{} adoption begins", path.name),
            probability: adoption_prob,
            impact: if adoption_prob > 0.7 {
                "smooth start expected".to_string()
            } else {
                "slow start likely".to_string()
            },
        });

        // Phase 2: Progress milestone — 50% mark
        let progress_days =
            (adoption_days * 2.5 / avg_velocity.max(0.05)).clamp(adoption_days + 3.0, 90.0);
        let progress_prob = (adoption_prob * 0.85 - risk * 0.1).clamp(0.15, 0.9);
        events.push(ProjectedEvent {
            time_offset_days: progress_days,
            event: format!("{} reaches midpoint", path.name),
            probability: progress_prob,
            impact: "execution cadence established".to_string(),
        });

        // Phase 3: Stabilization / completion
        let stable_days = (progress_days * 1.8).clamp(progress_days + 5.0, 180.0);
        let stable_prob = (progress_prob * 0.8).clamp(0.1, 0.85);
        events.push(ProjectedEvent {
            time_offset_days: stable_days,
            event: format!("{} stabilizes", path.name),
            probability: stable_prob,
            impact: if risk > 0.6 {
                "high-risk delivery, monitor closely".to_string()
            } else {
                "delivery quality on track".to_string()
            },
        });

        events
    }

    fn project_final_state(&self, _decision: &Decision, path: &DecisionPath) -> String {
        format!("{} selected with moderate confidence", path.name)
    }

    fn calculate_projection_confidence(&self, decision: &Decision) -> f64 {
        (0.5 + decision.reasoning.confidence * 0.4).clamp(0.1, 1.0)
    }

    fn project_path_timeline(&self, path: &DecisionPath) -> Vec<ProjectedEvent> {
        vec![ProjectedEvent {
            time_offset_days: 14.0,
            event: format!("{} key milestone", path.name),
            probability: 0.6,
            impact: "roadmap shift".to_string(),
        }]
    }

    fn project_path_final_state(&self, path: &DecisionPath) -> String {
        format!("Path {} likely reaches usable state", path.name)
    }

    fn assess_path_risk(&self, path: &DecisionPath) -> String {
        let risk = path.estimated_risk.unwrap_or(0.5);
        if risk > 0.7 {
            "high".to_string()
        } else if risk > 0.4 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    fn assess_path_opportunity(&self, path: &DecisionPath) -> String {
        if path.pros.len() >= path.cons.len() {
            "favorable".to_string()
        } else {
            "constrained".to_string()
        }
    }

    // --- Missing query operations (SPEC-PART2) ---

    pub fn search_decisions(&self, query: &str) -> Vec<Decision> {
        let query_lower = query.to_lowercase();
        self.decision_store
            .values()
            .filter(|d| {
                d.question.question.to_lowercase().contains(&query_lower)
                    || d.question.context.to_lowercase().contains(&query_lower)
                    || d.reasoning.rationale.to_lowercase().contains(&query_lower)
                    || d.chosen
                        .as_ref()
                        .map(|p| {
                            p.name.to_lowercase().contains(&query_lower)
                                || p.description.to_lowercase().contains(&query_lower)
                        })
                        .unwrap_or(false)
                    || d.shadows.iter().any(|s| {
                        s.path.name.to_lowercase().contains(&query_lower)
                            || s.path.description.to_lowercase().contains(&query_lower)
                            || s.rejection_reason.to_lowercase().contains(&query_lower)
                    })
                    || d.consequences
                        .iter()
                        .any(|c| c.description.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    pub fn get_progress_forecast(&self, id: GoalId) -> Result<ProgressForecast> {
        let goal = self.goal_store.get(&id).ok_or(Error::GoalNotFound(id))?;

        let now = Timestamp::now();
        let velocity = goal.progress.velocity;
        let current = goal.progress.percentage;

        let mut milestones = Vec::new();
        let mut risk_factors = Vec::new();

        if velocity > 0.0 {
            for target in &[0.25, 0.5, 0.75, 1.0] {
                if current < *target {
                    let remaining = target - current;
                    let days_needed = remaining / velocity;
                    let est_at =
                        Timestamp::from_nanos(now.0 + (days_needed * 86_400.0 * 1e9) as i64);
                    let confidence = (1.0_f64 - remaining * 0.3).clamp(0.1, 0.95);
                    milestones.push(ForecastMilestone {
                        percentage: *target,
                        estimated_at: est_at,
                        confidence,
                    });
                }
            }
        }

        let estimated_completion = if velocity > 0.0 && current < 1.0 {
            let days_to_complete = (1.0 - current) / velocity;
            Some(Timestamp::from_nanos(
                now.0 + (days_to_complete * 86_400.0 * 1e9) as i64,
            ))
        } else if current >= 1.0 {
            goal.completed_at
        } else {
            None
        };

        // Check deadline risk
        if let (Some(deadline), Some(est)) = (goal.deadline, estimated_completion) {
            if est.0 > deadline.0 {
                risk_factors.push("Forecast exceeds deadline".to_string());
            }
        }

        if !goal.blockers.iter().all(|b| b.resolved_at.is_some()) {
            risk_factors.push("Active blockers may slow progress".to_string());
        }

        if goal.feelings.neglect > 0.5 {
            risk_factors.push("High neglect may reduce velocity".to_string());
        }

        if goal.physics.momentum < 0.2 {
            risk_factors.push("Low momentum suggests stalling".to_string());
        }

        let confidence = if velocity > 0.0 {
            (goal.feelings.confidence * 0.5 + goal.physics.momentum * 0.3 + 0.2).clamp(0.1, 0.95)
        } else {
            0.1
        };

        Ok(ProgressForecast {
            goal_id: id,
            current_percentage: current,
            current_velocity: velocity,
            projected_milestones: milestones,
            estimated_completion,
            confidence,
            risk_factors,
        })
    }

    pub fn get_momentum_report(&self) -> MomentumReport {
        let active_goals: Vec<&Goal> = self
            .goal_store
            .values()
            .filter(|g| matches!(g.status, GoalStatus::Active | GoalStatus::Blocked))
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

    pub fn get_gravity_field(&self) -> GravityField {
        let active_goals: Vec<&Goal> = self
            .goal_store
            .values()
            .filter(|g| matches!(g.status, GoalStatus::Active | GoalStatus::Blocked))
            .collect();

        let total = active_goals.len();

        let (weighted_urgency, weighted_priority, weighted_momentum) = if total > 0 {
            let u = active_goals
                .iter()
                .map(|g| g.feelings.urgency * g.physics.gravity)
                .sum::<f64>()
                / total as f64;
            let p = active_goals
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
            let m = active_goals
                .iter()
                .map(|g| g.physics.momentum * g.physics.gravity)
                .sum::<f64>()
                / total as f64;
            (u, p, m)
        } else {
            (0.0, 0.0, 0.0)
        };

        let mut wells: Vec<GravityWell> = active_goals
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
}
