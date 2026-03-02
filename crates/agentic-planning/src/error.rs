use crate::types::{
    CommitmentId, CommitmentStatus, DecisionId, DecisionStatus, DreamId, FederationId, GoalId,
    GoalStatus, PathId,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("goal not found: {0:?}")]
    GoalNotFound(GoalId),

    #[error("decision not found: {0:?}")]
    DecisionNotFound(DecisionId),

    #[error("commitment not found: {0:?}")]
    CommitmentNotFound(CommitmentId),

    #[error("dream not found: {0:?}")]
    DreamNotFound(DreamId),

    #[error("federation not found: {0:?}")]
    FederationNotFound(FederationId),

    #[error("soul archive not found: {0:?}")]
    SoulNotFound(GoalId),

    #[error("path not found: {0:?}")]
    PathNotFound(PathId),

    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: GoalStatus, to: GoalStatus },

    #[error("cannot complete goal in state {0:?}")]
    CannotComplete(GoalStatus),

    #[error("cannot break commitment in state {0:?}")]
    CannotBreak(CommitmentStatus),

    #[error("decision already crystallized")]
    AlreadyCrystallized,

    #[error("cannot recrystallize decision in state {0:?}")]
    CannotRecrystallize(DecisionStatus),

    #[error("cannot fulfill commitment in state {0:?}")]
    CannotFulfill(CommitmentStatus),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("invalid .aplan file")]
    InvalidFile,

    #[error("corrupted .aplan file: {0}")]
    CorruptedFile(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
