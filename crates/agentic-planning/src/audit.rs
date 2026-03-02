use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Timestamp;

/// Actions that can be recorded in the audit log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    StatusChange,
    Crystallize,
    Fulfill,
    Break,
}

/// The entity type that was acted upon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEntityType {
    Goal,
    Decision,
    Commitment,
    Dream,
    Federation,
}

/// A single audit log entry recording an operation on the planning engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: Timestamp,
    pub session_id: Uuid,
    pub operation: String,
    pub entity_type: AuditEntityType,
    pub entity_id: String,
    pub action: AuditAction,
    pub details: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

/// Append-only audit log for tracking all planning engine mutations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        session_id: Uuid,
        operation: impl Into<String>,
        entity_type: AuditEntityType,
        entity_id: impl Into<String>,
        action: AuditAction,
        success: bool,
        details: Option<String>,
        error: Option<String>,
    ) {
        self.entries.push(AuditEntry {
            timestamp: Timestamp::now(),
            session_id,
            operation: operation.into(),
            entity_type,
            entity_id: entity_id.into(),
            action,
            details,
            success,
            error,
        });
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries_for_entity(&self, entity_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.entity_id == entity_id)
            .collect()
    }

    pub fn entries_by_action(&self, action: AuditAction) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.action == action).collect()
    }

    pub fn entries_since(&self, since: Timestamp) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp.0 >= since.0)
            .collect()
    }

    pub fn failures(&self) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| !e.success).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_append_and_query() {
        let mut log = AuditLog::new();
        assert!(log.is_empty());

        let session = Uuid::new_v4();
        log.record(
            session,
            "create_goal",
            AuditEntityType::Goal,
            "goal-123",
            AuditAction::Create,
            true,
            Some("Created goal".into()),
            None,
        );

        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
        assert_eq!(log.entries_for_entity("goal-123").len(), 1);
        assert_eq!(log.entries_by_action(AuditAction::Create).len(), 1);
        assert!(log.failures().is_empty());
    }

    #[test]
    fn test_audit_log_failures() {
        let mut log = AuditLog::new();
        let session = Uuid::new_v4();

        log.record(
            session,
            "create_goal",
            AuditEntityType::Goal,
            "goal-1",
            AuditAction::Create,
            true,
            None,
            None,
        );

        log.record(
            session,
            "update_goal",
            AuditEntityType::Goal,
            "goal-2",
            AuditAction::Update,
            false,
            None,
            Some("Goal not found".into()),
        );

        assert_eq!(log.len(), 2);
        assert_eq!(log.failures().len(), 1);
        assert_eq!(log.failures()[0].entity_id, "goal-2");
    }

    #[test]
    fn test_audit_log_entries_since() {
        let mut log = AuditLog::new();
        let session = Uuid::new_v4();
        let before = Timestamp::now();

        log.record(
            session,
            "create_decision",
            AuditEntityType::Decision,
            "dec-1",
            AuditAction::Create,
            true,
            None,
            None,
        );

        log.record(
            session,
            "crystallize_decision",
            AuditEntityType::Decision,
            "dec-1",
            AuditAction::Crystallize,
            true,
            None,
            None,
        );

        assert_eq!(log.entries_since(before).len(), 2);
        assert_eq!(log.entries_by_action(AuditAction::Crystallize).len(), 1);
    }

    #[test]
    fn test_audit_entity_types_and_actions() {
        let mut log = AuditLog::new();
        let session = Uuid::new_v4();

        let actions = [
            (AuditAction::Create, AuditEntityType::Goal),
            (AuditAction::Read, AuditEntityType::Decision),
            (AuditAction::Update, AuditEntityType::Commitment),
            (AuditAction::Delete, AuditEntityType::Dream),
            (AuditAction::StatusChange, AuditEntityType::Goal),
            (AuditAction::Crystallize, AuditEntityType::Decision),
            (AuditAction::Fulfill, AuditEntityType::Commitment),
            (AuditAction::Break, AuditEntityType::Federation),
        ];

        for (i, (action, entity_type)) in actions.iter().enumerate() {
            log.record(
                session,
                format!("op_{}", i),
                *entity_type,
                format!("id-{}", i),
                *action,
                true,
                None,
                None,
            );
        }

        assert_eq!(log.len(), 8);
        assert_eq!(log.entries_by_action(AuditAction::Create).len(), 1);
        assert_eq!(log.entries_by_action(AuditAction::Fulfill).len(), 1);
    }
}
