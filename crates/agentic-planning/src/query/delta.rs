//! Delta retrieval — change-proportional queries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change<T> {
    pub id: String,
    pub change_type: ChangeType,
    pub timestamp: i64,
    pub value: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaResult<T> {
    pub changes: Vec<Change<T>>,
    pub from_version: u64,
    pub to_version: u64,
    pub has_more: bool,
}

impl<T> DeltaResult<T> {
    pub fn empty(version: u64) -> Self {
        Self { changes: Vec::new(), from_version: version, to_version: version, has_more: false }
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }
}

#[derive(Debug, Clone)]
pub struct VersionedState {
    version: u64,
    timestamps: Vec<(String, i64, ChangeType)>,
}

impl VersionedState {
    pub fn new() -> Self {
        Self { version: 0, timestamps: Vec::new() }
    }

    pub fn record_change(&mut self, id: impl Into<String>, change_type: ChangeType) {
        self.version += 1;
        let ts = chrono::Utc::now().timestamp_micros();
        self.timestamps.push((id.into(), ts, change_type));
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn changes_since(&self, since_ts: i64) -> Vec<&(String, i64, ChangeType)> {
        self.timestamps.iter().filter(|(_, ts, _)| *ts > since_ts).collect()
    }

    pub fn changes_since_version(&self, since_version: u64) -> Vec<&(String, i64, ChangeType)> {
        if since_version as usize >= self.timestamps.len() {
            return Vec::new();
        }
        self.timestamps[since_version as usize..].iter().collect()
    }

    pub fn last_change_timestamp(&self) -> i64 {
        self.timestamps.last().map(|(_, ts, _)| *ts).unwrap_or(0)
    }

    pub fn is_unchanged_since(&self, since_version: u64) -> bool {
        self.version <= since_version
    }
}

impl Default for VersionedState {
    fn default() -> Self {
        Self::new()
    }
}
