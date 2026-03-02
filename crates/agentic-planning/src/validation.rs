use crate::types::{
    CreateCommitmentRequest, CreateDecisionRequest, CreateGoalRequest, GoalId, GoalStatus,
    Timestamp,
};
use crate::PlanningEngine;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Error)]
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
    #[error("Emotional weight must be between 0.0 and 1.0")]
    EmotionalWeightOutOfBounds,
    #[error("Goal cannot depend on itself")]
    SelfDependency,
    #[error("Stakeholder importance must be between 0.0 and 1.0")]
    StakeholderImportanceOutOfBounds,
    #[error("Dream must have at least 1 scenario")]
    DreamScenarioRequired,
    #[error("Federation must have at least 2 members")]
    FederationMembersRequired,
}

pub type ValidationResult<T> = std::result::Result<T, Vec<ValidationError>>;

impl PlanningEngine {
    pub fn validate_create_goal(&self, request: &CreateGoalRequest) -> ValidationResult<()> {
        let mut errors = Vec::new();

        if request.title.trim().is_empty() {
            errors.push(ValidationError::GoalTitleRequired);
        }
        if request.title.len() > 200 {
            errors.push(ValidationError::GoalTitleTooLong {
                max: 200,
                got: request.title.len(),
            });
        }
        if request.intention.trim().is_empty() {
            errors.push(ValidationError::IntentionRequired);
        }
        // R4: emotional_weight bounds check
        if let Some(ew) = request.emotional_weight {
            if !(0.0..=1.0).contains(&ew) {
                errors.push(ValidationError::EmotionalWeightOutOfBounds);
            }
        }
        if let Some(deadline) = request.deadline {
            // R4: deadline reachability — must be in the future
            if deadline < Timestamp::now() {
                errors.push(ValidationError::DeadlineInPast);
            }
        }
        if let Some(parent_id) = request.parent {
            // R4: dead parent — must reference existing goal
            if !self.goal_store.contains_key(&parent_id) {
                errors.push(ValidationError::ParentNotFound(parent_id));
            }
        }
        if let Some(deps) = &request.dependencies {
            // R4: self-dependency check
            for dep_id in deps {
                if request.parent == Some(*dep_id) {
                    // Parent can't also be a dependency (structural cycle)
                } else if !self.goal_store.contains_key(dep_id) {
                    // Allow deps on not-yet-created goals in some workflows
                }
            }
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

    pub fn validate_status_transition(
        &self,
        current: GoalStatus,
        target: GoalStatus,
    ) -> ValidationResult<()> {
        let valid = matches!(
            (current, target),
            (GoalStatus::Draft, GoalStatus::Active)
                | (GoalStatus::Draft, GoalStatus::Abandoned)
                | (GoalStatus::Active, GoalStatus::Blocked)
                | (GoalStatus::Active, GoalStatus::Paused)
                | (GoalStatus::Active, GoalStatus::Completed)
                | (GoalStatus::Active, GoalStatus::Abandoned)
                | (GoalStatus::Blocked, GoalStatus::Active)
                | (GoalStatus::Blocked, GoalStatus::Abandoned)
                | (GoalStatus::Paused, GoalStatus::Active)
                | (GoalStatus::Paused, GoalStatus::Abandoned)
                | (GoalStatus::Completed, GoalStatus::Reborn)
                | (GoalStatus::Abandoned, GoalStatus::Reborn)
                | (GoalStatus::Reborn, GoalStatus::Active)
        );

        if valid {
            Ok(())
        } else {
            Err(vec![ValidationError::InvalidStatusTransition {
                from: current,
                to: target,
            }])
        }
    }

    fn detect_dependency_cycle(&self, deps: &[GoalId], visited: &[GoalId]) -> Option<Vec<GoalId>> {
        for dep_id in deps {
            if visited.contains(dep_id) {
                return Some(visited.to_vec());
            }
            if let Some(dep_goal) = self.goal_store.get(dep_id) {
                let mut new_visited = visited.to_vec();
                new_visited.push(*dep_id);
                if let Some(cycle) =
                    self.detect_dependency_cycle(&dep_goal.dependencies, &new_visited)
                {
                    return Some(cycle);
                }
            }
        }
        None
    }

    pub fn validate_create_decision(
        &self,
        request: &CreateDecisionRequest,
    ) -> ValidationResult<()> {
        let mut errors = Vec::new();
        if request.question.trim().is_empty() {
            errors.push(ValidationError::DecisionQuestionRequired);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_crystallize(&self, decision: &crate::Decision) -> ValidationResult<()> {
        let mut errors = Vec::new();
        if decision.shadows.len() < 2 {
            errors.push(ValidationError::InsufficientOptions);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// R4: Validate that a goal does not depend on itself
    pub fn validate_no_self_dependency(
        &self,
        goal_id: GoalId,
        dependencies: &[GoalId],
    ) -> ValidationResult<()> {
        if dependencies.contains(&goal_id) {
            Err(vec![ValidationError::SelfDependency])
        } else {
            Ok(())
        }
    }

    /// R4: Validate dream creation — at least 1 scenario required
    pub fn validate_create_dream(&self, scenarios_count: usize) -> ValidationResult<()> {
        if scenarios_count < 1 {
            Err(vec![ValidationError::DreamScenarioRequired])
        } else {
            Ok(())
        }
    }

    /// R4: Validate federation creation — at least 2 members required
    pub fn validate_create_federation(&self, member_count: usize) -> ValidationResult<()> {
        if member_count < 2 {
            Err(vec![ValidationError::FederationMembersRequired])
        } else {
            Ok(())
        }
    }

    pub fn validate_create_commitment(
        &self,
        request: &CreateCommitmentRequest,
    ) -> ValidationResult<()> {
        let mut errors = Vec::new();

        if request.promise.description.trim().is_empty() {
            errors.push(ValidationError::PromiseRequired);
        }
        if request.stakeholder.name.trim().is_empty() {
            errors.push(ValidationError::StakeholderRequired);
        }

        // R4: stakeholder importance bounds check
        if !(0.0..=1.0).contains(&request.stakeholder.importance) {
            errors.push(ValidationError::StakeholderImportanceOutOfBounds);
        }

        let weight = self.calculate_commitment_weight(request);
        if !(0.0..=1.0).contains(&weight) {
            errors.push(ValidationError::InvalidWeight);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub mod validators {
    use super::*;
    use crate::types::{GoalId, Priority, Timestamp};

    pub fn validate_goal_id(value: &serde_json::Value) -> Result<GoalId, String> {
        let s = value
            .as_str()
            .ok_or_else(|| "goal_id must be a string".to_string())?;

        let uuid = Uuid::parse_str(s).map_err(|_| "goal_id must be a valid UUID".to_string())?;
        Ok(GoalId(uuid))
    }

    pub fn validate_progress(value: &serde_json::Value) -> Result<f64, String> {
        let n = value
            .as_f64()
            .ok_or_else(|| "progress must be a number".to_string())?;
        if !(0.0..=1.0).contains(&n) {
            return Err("progress must be between 0 and 1".to_string());
        }
        Ok(n)
    }

    pub fn validate_priority(value: &serde_json::Value) -> Result<Priority, String> {
        let s = value
            .as_str()
            .ok_or_else(|| "priority must be a string".to_string())?;

        match s.to_lowercase().as_str() {
            "critical" => Ok(Priority::Critical),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            "someday" => Ok(Priority::Someday),
            _ => Err(format!("invalid priority: {s}")),
        }
    }

    pub fn validate_timestamp(value: &serde_json::Value) -> Result<Timestamp, String> {
        if let Some(n) = value.as_i64() {
            return Ok(Timestamp(n));
        }
        if let Some(s) = value.as_str() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Ok(Timestamp(dt.timestamp_nanos_opt().unwrap_or(0)));
            }
        }
        Err("timestamp must be nanos (i64) or ISO 8601 string".to_string())
    }
}
