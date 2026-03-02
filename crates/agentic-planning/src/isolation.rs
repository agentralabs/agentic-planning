//! Per-project identity isolation.
//!
//! Each project gets a deterministic `ProjectId` derived from its canonical
//! filesystem path. Two folders with the same name but different parent paths
//! always produce different IDs.

use blake3::Hasher;
use std::fmt;
use std::path::{Path, PathBuf};

/// A 32-byte project identity derived from the canonical path via blake3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId([u8; 32]);

impl ProjectId {
    /// Raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// First 16 hex characters — useful for short display.
    pub fn short(&self) -> String {
        hex::encode(&self.0[..8])
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

/// Derive a deterministic project identity from a filesystem path.
///
/// The path is canonicalized first so symlinks resolve to the same identity.
/// If canonicalization fails (path doesn't exist yet), we fall back to the
/// absolute path.
pub fn project_identity(project_path: &Path) -> ProjectId {
    let canonical = project_path.canonicalize().unwrap_or_else(|_| {
        std::path::absolute(project_path).unwrap_or(project_path.to_path_buf())
    });

    let mut hasher = Hasher::new();
    hasher.update(b"agentic-planning-project-v1:");
    hasher.update(canonical.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    ProjectId(*hash.as_bytes())
}

/// Return the cache directory for a given project identity.
///
/// Layout: `<base_dir>/<short-id>/`
pub fn cache_dir(base: &Path, id: &ProjectId) -> PathBuf {
    base.join(id.short())
}

/// Resolve the graph file for a project, returning an error if not found.
/// Never falls back to "latest" or any other project.
pub fn resolve_graph(base: &Path, id: &ProjectId) -> Result<PathBuf, IsolationError> {
    let dir = cache_dir(base, id);
    let graph_file = dir.join("planning.aplan");
    if graph_file.exists() {
        Ok(graph_file)
    } else {
        Err(IsolationError::GraphNotFound {
            project_id: *id,
            expected_path: graph_file,
        })
    }
}

#[derive(Debug)]
pub enum IsolationError {
    GraphNotFound {
        project_id: ProjectId,
        expected_path: PathBuf,
    },
}

impl fmt::Display for IsolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsolationError::GraphNotFound {
                project_id,
                expected_path,
            } => write!(
                f,
                "graph not found for project {}: {}",
                project_id.short(),
                expected_path.display()
            ),
        }
    }
}

impl std::error::Error for IsolationError {}

// --- hex helper (avoids adding a `hex` crate dependency) ---

mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn same_name_different_path() {
        let a = project_identity(&PathBuf::from("/home/alice/projects/myapp"));
        let b = project_identity(&PathBuf::from("/home/bob/projects/myapp"));
        assert_ne!(a, b, "same folder name under different parents must differ");
    }

    #[test]
    fn deterministic() {
        let path = PathBuf::from("/tmp/test-deterministic-planning");
        let id1 = project_identity(&path);
        let id2 = project_identity(&path);
        assert_eq!(id1, id2);
    }

    #[test]
    fn short_is_16_chars() {
        let id = project_identity(&PathBuf::from("/whatever"));
        assert_eq!(id.short().len(), 16);
    }

    #[test]
    fn display_is_64_chars() {
        let id = project_identity(&PathBuf::from("/whatever"));
        assert_eq!(id.to_string().len(), 64);
    }

    #[test]
    fn graph_not_found_errors() {
        let base = PathBuf::from("/tmp/nonexistent-aplan-test");
        let id = project_identity(&PathBuf::from("/some/project"));
        assert!(resolve_graph(&base, &id).is_err());
    }

    #[test]
    fn stress_multi_project_isolation() {
        let ids: Vec<ProjectId> = (0..10)
            .map(|i| project_identity(&PathBuf::from(format!("/workspace/team{}/myapp", i))))
            .collect();

        // All must be unique
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j], "project {} and {} collided", i, j);
            }
        }
    }
}
