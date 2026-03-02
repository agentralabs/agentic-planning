//! Concurrent startup locking with stale-lock recovery, and file-level locking
//! for safe concurrent writes to `.aplan` files.
//!
//! Only one instance of a named lock can be held at a time. Locks older than
//! `STALE_THRESHOLD` whose owning process is no longer alive are automatically
//! recovered.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How old a lock file must be before we check if the owning process is alive.
const STALE_THRESHOLD: Duration = Duration::from_secs(5 * 60);

/// How long to wait between retry attempts when acquiring a file lock.
const FILE_LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(50);

/// Maximum time to wait for a file lock before giving up.
const FILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// A PID-based lock file that auto-cleans on drop.
pub struct StartupLock {
    path: PathBuf,
}

impl StartupLock {
    /// Acquire a named lock. Returns `Err` if another live process holds it.
    pub fn acquire(name: &str) -> Result<Self, LockError> {
        let dir = lock_dir();
        fs::create_dir_all(&dir).map_err(LockError::Io)?;

        let path = dir.join(format!("agentic-planning-{}.lock", name));
        let my_pid = std::process::id();

        // Check existing lock
        if path.exists() {
            let contents = fs::read_to_string(&path).unwrap_or_default();
            if let Ok(existing_pid) = contents.trim().parse::<u32>() {
                if existing_pid == my_pid {
                    // We already hold it — re-acquire
                    return Ok(Self { path });
                }

                let metadata = fs::metadata(&path).ok();
                let age = metadata
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| SystemTime::now().duration_since(t).ok());

                let is_stale = age.map(|a| a > STALE_THRESHOLD).unwrap_or(true);

                if is_stale && !is_process_alive(existing_pid) {
                    // Stale lock from dead process — recover
                    let _ = fs::remove_file(&path);
                } else if is_process_alive(existing_pid) {
                    return Err(LockError::AlreadyHeld {
                        name: name.to_string(),
                        pid: existing_pid,
                    });
                } else {
                    // Process is dead but lock is recent — still recover
                    let _ = fs::remove_file(&path);
                }
            } else {
                // Corrupted lock file — remove
                let _ = fs::remove_file(&path);
            }
        }

        // Write our PID
        let mut f = fs::File::create(&path).map_err(LockError::Io)?;
        write!(f, "{}", my_pid).map_err(LockError::Io)?;
        f.sync_all().map_err(LockError::Io)?;

        // Verify we actually got it (race protection)
        let verify = fs::read_to_string(&path).unwrap_or_default();
        if verify.trim() != my_pid.to_string() {
            return Err(LockError::RaceCondition {
                name: name.to_string(),
            });
        }

        Ok(Self { path })
    }

    /// Touch the lock file to prevent stale detection.
    pub fn touch(&self) -> io::Result<()> {
        let pid = std::process::id();
        fs::write(&self.path, pid.to_string())
    }

    /// Path to the lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for StartupLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug)]
pub enum LockError {
    AlreadyHeld { name: String, pid: u32 },
    RaceCondition { name: String },
    Io(io::Error),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::AlreadyHeld { name, pid } => {
                write!(f, "lock '{}' already held by PID {}", name, pid)
            }
            LockError::RaceCondition { name } => {
                write!(f, "race condition acquiring lock '{}'", name)
            }
            LockError::Io(e) => write!(f, "lock I/O error: {}", e),
        }
    }
}

impl std::error::Error for LockError {}

/// File-level lock for `.aplan` files. Creates a `.aplan.lock` sidecar that
/// prevents concurrent writes to the same planning file.
///
/// The lock is held for the duration of the write and released on drop.
pub struct FileLock {
    path: PathBuf,
}

impl FileLock {
    /// Acquire a file-level lock for the given `.aplan` path.
    ///
    /// Creates a `.aplan.lock` sidecar file containing the current PID.
    /// Retries with backoff up to `FILE_LOCK_TIMEOUT`. Stale locks from
    /// dead processes are automatically recovered.
    pub fn acquire(aplan_path: &Path) -> Result<Self, LockError> {
        let lock_path = lock_path_for(aplan_path);

        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(LockError::Io)?;
        }

        let my_pid = std::process::id();
        let start = std::time::Instant::now();

        loop {
            match Self::try_acquire(&lock_path, my_pid) {
                Ok(lock) => return Ok(lock),
                Err(LockError::AlreadyHeld { .. }) if start.elapsed() < FILE_LOCK_TIMEOUT => {
                    std::thread::sleep(FILE_LOCK_RETRY_INTERVAL);
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn try_acquire(lock_path: &Path, my_pid: u32) -> Result<Self, LockError> {
        if lock_path.exists() {
            let contents = fs::read_to_string(lock_path).unwrap_or_default();
            if let Ok(existing_pid) = contents.trim().parse::<u32>() {
                if existing_pid == my_pid {
                    // We already hold it
                    return Ok(Self {
                        path: lock_path.to_path_buf(),
                    });
                }

                if !is_process_alive(existing_pid) {
                    // Dead process — recover
                    let _ = fs::remove_file(lock_path);
                } else {
                    // Check age — stale locks from hung processes
                    let age = fs::metadata(lock_path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| SystemTime::now().duration_since(t).ok());

                    let is_stale = age.map(|a| a > STALE_THRESHOLD).unwrap_or(false);
                    if is_stale {
                        let _ = fs::remove_file(lock_path);
                    } else {
                        return Err(LockError::AlreadyHeld {
                            name: lock_path.display().to_string(),
                            pid: existing_pid,
                        });
                    }
                }
            } else {
                // Corrupted lock — remove
                let _ = fs::remove_file(lock_path);
            }
        }

        // Write our PID
        let mut f = fs::File::create(lock_path).map_err(LockError::Io)?;
        write!(f, "{}", my_pid).map_err(LockError::Io)?;
        f.sync_all().map_err(LockError::Io)?;

        // Verify we got it
        let verify = fs::read_to_string(lock_path).unwrap_or_default();
        if verify.trim() != my_pid.to_string() {
            return Err(LockError::RaceCondition {
                name: lock_path.display().to_string(),
            });
        }

        Ok(Self {
            path: lock_path.to_path_buf(),
        })
    }

    /// Path to the lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Compute the `.aplan.lock` sidecar path for a given `.aplan` file.
pub fn lock_path_for(aplan_path: &Path) -> PathBuf {
    let mut lock = aplan_path.to_path_buf();
    lock.set_extension("aplan.lock");
    lock
}

fn lock_dir() -> PathBuf {
    std::env::temp_dir().join("agentic-planning-locks")
}

/// Check if a process with the given PID is alive.
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    // kill(pid, 0) checks existence without sending a signal
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_alive(_pid: u32) -> bool {
    // Conservative: assume alive on non-Unix platforms
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release() {
        let lock = StartupLock::acquire("test-acquire").unwrap();
        assert!(lock.path().exists());
        let path = lock.path().to_path_buf();
        drop(lock);
        assert!(!path.exists());
    }

    #[test]
    fn same_process_reacquire() {
        let _lock1 = StartupLock::acquire("test-reacquire").unwrap();
        // Same process can re-acquire
        let _lock2 = StartupLock::acquire("test-reacquire").unwrap();
    }

    #[test]
    fn touch_refreshes() {
        let lock = StartupLock::acquire("test-touch").unwrap();
        assert!(lock.touch().is_ok());
    }

    #[test]
    fn file_lock_acquire_and_release() {
        let dir = tempfile::tempdir().unwrap();
        let aplan = dir.path().join("test.aplan");
        std::fs::write(&aplan, "{}").unwrap();

        let lock = FileLock::acquire(&aplan).unwrap();
        let lock_path = lock.path().to_path_buf();
        assert!(lock_path.exists());
        drop(lock);
        assert!(!lock_path.exists());
    }

    #[test]
    fn file_lock_same_process_reacquire() {
        let dir = tempfile::tempdir().unwrap();
        let aplan = dir.path().join("reacq.aplan");
        std::fs::write(&aplan, "{}").unwrap();

        let _lock1 = FileLock::acquire(&aplan).unwrap();
        // Same process can re-acquire
        let _lock2 = FileLock::acquire(&aplan).unwrap();
    }

    #[test]
    fn file_lock_dead_process_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let aplan = dir.path().join("dead.aplan");
        std::fs::write(&aplan, "{}").unwrap();

        let lock_path = lock_path_for(&aplan);
        // Write a lock from an unlikely PID
        std::fs::write(&lock_path, "999999999").unwrap();

        // Should recover since that PID is dead
        let lock = FileLock::acquire(&aplan).unwrap();
        assert!(lock.path().exists());
    }

    #[test]
    fn lock_path_for_correctness() {
        let p = std::path::PathBuf::from("/tmp/test.aplan");
        let lp = lock_path_for(&p);
        assert_eq!(lp.to_str().unwrap(), "/tmp/test.aplan.lock");
    }

    #[test]
    fn stale_lock_recovery() {
        let dir = lock_dir();
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("agentic-planning-test-stale.lock");

        // Write a fake lock from PID 1 (init — always alive on Linux, but
        // we set the modified time far in the past)
        fs::write(&path, "999999999").unwrap(); // unlikely PID

        // Should recover
        let lock = StartupLock::acquire("test-stale").unwrap();
        assert!(lock.path().exists());
    }
}
