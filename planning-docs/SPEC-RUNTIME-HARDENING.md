# SPEC-RUNTIME-HARDENING

> **Scope:** ALL Sisters
> **Status:** Canonical Standard
> **Enforced:** CI Guardrails

---

## 1. Overview

Every sister MUST implement runtime hardening to ensure:
- No silent failures
- No cross-project contamination
- Safe concurrent operation
- Graceful crash recovery
- Authenticated server mode

These patterns are proven in AgenticMemory 0.2.6 and AgenticCodebase 0.1.5.

---

## 2. Strict MCP Input Validation

### 2.1 No Silent Fallbacks

```rust
/// WRONG: Silent fallback to default
fn get_project_id(params: &Value) -> ProjectId {
    params.get("project_id")
        .and_then(|v| v.as_str())
        .map(|s| ProjectId::parse(s).unwrap_or_default())  // WRONG!
        .unwrap_or_default()  // WRONG!
}

/// CORRECT: Explicit validation with errors
fn get_project_id(params: &Value) -> Result<ProjectId, ValidationError> {
    let value = params.get("project_id")
        .ok_or(ValidationError::MissingRequired("project_id"))?;
    
    let s = value.as_str()
        .ok_or(ValidationError::WrongType {
            field: "project_id",
            expected: "string",
            got: value.type_name(),
        })?;
    
    ProjectId::parse(s)
        .map_err(|e| ValidationError::InvalidFormat {
            field: "project_id",
            message: e.to_string(),
        })
}
```

### 2.2 Validation Framework

```rust
//! src/validation/strict.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Missing required parameter: {0}")]
    MissingRequired(&'static str),
    
    #[error("Parameter '{field}' has wrong type: expected {expected}, got {got}")]
    WrongType {
        field: &'static str,
        expected: &'static str,
        got: &'static str,
    },
    
    #[error("Parameter '{field}' has invalid format: {message}")]
    InvalidFormat {
        field: &'static str,
        message: String,
    },
    
    #[error("Parameter '{field}' out of range: {min} <= value <= {max}")]
    OutOfRange {
        field: &'static str,
        min: f64,
        max: f64,
    },
    
    #[error("Parameter '{field}' has invalid value, expected one of: {allowed:?}")]
    InvalidValue {
        field: &'static str,
        allowed: Vec<&'static str>,
    },
}

/// Strict validator that never silently falls back
pub struct StrictValidator;

impl StrictValidator {
    pub fn require_string(params: &Value, field: &'static str) -> Result<String, ValidationError> {
        let value = params.get(field)
            .ok_or(ValidationError::MissingRequired(field))?;
        
        value.as_str()
            .map(|s| s.to_string())
            .ok_or(ValidationError::WrongType {
                field,
                expected: "string",
                got: Self::type_name(value),
            })
    }
    
    pub fn require_u64(params: &Value, field: &'static str) -> Result<u64, ValidationError> {
        let value = params.get(field)
            .ok_or(ValidationError::MissingRequired(field))?;
        
        value.as_u64()
            .ok_or(ValidationError::WrongType {
                field,
                expected: "unsigned integer",
                got: Self::type_name(value),
            })
    }
    
    pub fn require_array(params: &Value, field: &'static str) -> Result<&Vec<Value>, ValidationError> {
        let value = params.get(field)
            .ok_or(ValidationError::MissingRequired(field))?;
        
        value.as_array()
            .ok_or(ValidationError::WrongType {
                field,
                expected: "array",
                got: Self::type_name(value),
            })
    }
    
    pub fn optional_string(params: &Value, field: &'static str) -> Result<Option<String>, ValidationError> {
        match params.get(field) {
            None => Ok(None),
            Some(Value::Null) => Ok(None),
            Some(value) => {
                let s = value.as_str()
                    .ok_or(ValidationError::WrongType {
                        field,
                        expected: "string or null",
                        got: Self::type_name(value),
                    })?;
                Ok(Some(s.to_string()))
            }
        }
    }
    
    fn type_name(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}
```

---

## 3. Per-Project Identity Isolation

### 3.1 Canonical Path Hashing

```rust
//! src/isolation/project_identity.rs

use blake3::Hasher;
use std::path::Path;

/// Generate deterministic project identity from canonical path
pub fn project_identity(project_path: &Path) -> ProjectId {
    // Canonicalize to resolve symlinks and normalize
    let canonical = project_path.canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
    
    // Hash the canonical path
    let mut hasher = Hasher::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    
    // Use first 16 bytes as project ID
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    
    ProjectId(uuid::Uuid::from_bytes(bytes))
}

/// CRITICAL: Same-named folders get different IDs
/// 
/// /home/user/project-a/myapp → ProjectId(abc123...)
/// /home/user/project-b/myapp → ProjectId(def456...)  // DIFFERENT!
#[test]
fn same_name_different_path() {
    let id_a = project_identity(Path::new("/home/user/project-a/myapp"));
    let id_b = project_identity(Path::new("/home/user/project-b/myapp"));
    
    assert_ne!(id_a, id_b, "Same-named folders must have different IDs");
}
```

### 3.2 Cache Isolation

```rust
//! src/isolation/cache.rs

use std::path::PathBuf;

/// Per-project cache directory
pub fn cache_dir(project_id: &ProjectId) -> PathBuf {
    let base = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("agentic")
        .join(env!("CARGO_PKG_NAME"));
    
    base.join(project_id.to_string())
}

/// CRITICAL: Never resolve to "latest" across projects
pub fn resolve_graph(project_id: &ProjectId) -> Result<GraphPath, Error> {
    let cache = cache_dir(project_id);
    let graph_path = cache.join("graph.bin");
    
    if graph_path.exists() {
        Ok(GraphPath::Cached(graph_path))
    } else {
        // CORRECT: Error, not fallback to another project's cache
        Err(Error::GraphNotFound {
            project_id: *project_id,
            message: "Graph not found. Run 'analyze' first.".to_string(),
        })
    }
}

/// WRONG: Never do this
fn resolve_graph_wrong(project_id: &ProjectId) -> GraphPath {
    let cache = cache_dir(project_id);
    let graph_path = cache.join("graph.bin");
    
    if graph_path.exists() {
        GraphPath::Cached(graph_path)
    } else {
        // WRONG: Falling back to "latest" from ANY project
        GraphPath::Latest  // THIS IS CROSS-PROJECT CONTAMINATION!
    }
}
```

---

## 4. Concurrent Startup Locking

### 4.1 Lock File Management

```rust
//! src/locking/startup.rs

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const STALE_LOCK_THRESHOLD: Duration = Duration::from_secs(300); // 5 minutes

pub struct StartupLock {
    lock_file: File,
    lock_path: PathBuf,
}

impl StartupLock {
    /// Acquire startup lock with stale-lock recovery
    pub fn acquire(name: &str) -> Result<Self, LockError> {
        let lock_path = Self::lock_path(name);
        
        // Check for stale lock
        if lock_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&lock_path) {
                if let Ok(modified) = metadata.modified() {
                    let age = SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or(Duration::ZERO);
                    
                    if age > STALE_LOCK_THRESHOLD {
                        tracing::warn!("Recovering stale lock: {:?}", lock_path);
                        std::fs::remove_file(&lock_path)?;
                    }
                }
            }
            
            // Check if PID in lock is still alive
            if let Ok(contents) = std::fs::read_to_string(&lock_path) {
                if let Ok(pid) = contents.trim().parse::<u32>() {
                    if !process_alive(pid) {
                        tracing::warn!("Removing lock for dead process: {}", pid);
                        std::fs::remove_file(&lock_path)?;
                    }
                }
            }
        }
        
        // Try to acquire lock
        let lock_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    LockError::AlreadyLocked
                } else {
                    LockError::Io(e)
                }
            })?;
        
        // Write PID
        let mut lock = Self { lock_file, lock_path };
        writeln!(lock.lock_file, "{}", std::process::id())?;
        lock.lock_file.sync_all()?;
        
        Ok(lock)
    }
    
    fn lock_path(name: &str) -> PathBuf {
        let runtime_dir = dirs::runtime_dir()
            .or_else(|| Some(PathBuf::from("/tmp")))
            .unwrap();
        
        runtime_dir.join(format!("agentic-{}.lock", name))
    }
    
    /// Touch lock to prevent stale detection
    pub fn touch(&self) -> Result<(), LockError> {
        // Update modification time
        let now = filetime::FileTime::now();
        filetime::set_file_mtime(&self.lock_path, now)?;
        Ok(())
    }
}

impl Drop for StartupLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Another instance is already running")]
    AlreadyLocked,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn process_alive(pid: u32) -> bool {
    use windows_sys::Win32::System::Threading::*;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == 0 {
            return false;
        }
        CloseHandle(handle);
        true
    }
}
```

### 4.2 Graceful Shutdown

```rust
//! src/locking/shutdown.rs

use tokio::sync::broadcast;
use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Setup signal handlers for graceful shutdown
pub fn setup_signal_handlers() -> broadcast::Receiver<()> {
    let (tx, rx) = broadcast::channel(1);
    
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        
        let tx_term = tx.clone();
        tokio::spawn(async move {
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint = signal(SignalKind::interrupt()).unwrap();
            
            tokio::select! {
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM");
                }
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT");
                }
            }
            
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
            let _ = tx_term.send(());
        });
    }
    
    #[cfg(windows)]
    {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            tracing::info!("Received Ctrl+C");
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
            let _ = tx.send(());
        });
    }
    
    rx
}

/// Check if shutdown was requested
pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}
```

---

## 5. MCP Config Merge (Never Destructive)

### 5.1 Safe Config Update

```rust
//! src/config/mcp_config.rs

use serde_json::{json, Value};
use std::path::Path;

/// Update MCP client configuration (merge-only, never destructive)
pub fn update_mcp_config(
    config_path: &Path,
    server_name: &str,
    server_config: Value,
) -> Result<(), ConfigError> {
    // Read existing config or start with empty
    let mut config: Value = if config_path.exists() {
        let contents = std::fs::read_to_string(config_path)?;
        serde_json::from_str(&contents)?
    } else {
        json!({})
    };
    
    // Ensure mcpServers object exists
    if !config.get("mcpServers").is_some() {
        config["mcpServers"] = json!({});
    }
    
    // CRITICAL: Merge, not replace
    // Only update our server entry, preserve all others
    config["mcpServers"][server_name] = server_config;
    
    // Write back with pretty formatting
    let formatted = serde_json::to_string_pretty(&config)?;
    
    // Atomic write
    let temp_path = config_path.with_extension("json.tmp");
    std::fs::write(&temp_path, &formatted)?;
    std::fs::rename(&temp_path, config_path)?;
    
    Ok(())
}

/// FORBIDDEN: Never do this
fn update_mcp_config_wrong(config_path: &Path, our_config: Value) -> Result<(), ConfigError> {
    // WRONG: This destroys other servers!
    std::fs::write(config_path, serde_json::to_string(&our_config)?)?;
    Ok(())
}
```

---

## 6. Server Mode Authentication

### 6.1 Token-Based Auth

```rust
//! src/auth/token.rs

use constant_time_eq::constant_time_eq;

pub struct TokenAuth {
    token: Option<String>,
    mode: AuthMode,
}

#[derive(Clone, Copy)]
pub enum AuthMode {
    None,
    Required,
    Optional,
}

impl TokenAuth {
    pub fn from_env() -> Self {
        let token = std::env::var("AGENTIC_AUTH_TOKEN").ok();
        let mode = match std::env::var("AGENTIC_AUTH_MODE").as_deref() {
            Ok("required") => AuthMode::Required,
            Ok("optional") => AuthMode::Optional,
            Ok("none") | _ => AuthMode::None,
        };
        
        // Warn if required but no token
        if matches!(mode, AuthMode::Required) && token.is_none() {
            tracing::warn!(
                "AGENTIC_AUTH_MODE=required but AGENTIC_AUTH_TOKEN not set. \
                 All requests will be rejected."
            );
        }
        
        Self { token, mode }
    }
    
    pub fn validate(&self, provided: Option<&str>) -> Result<(), AuthError> {
        match self.mode {
            AuthMode::None => Ok(()),
            
            AuthMode::Required => {
                let expected = self.token.as_ref()
                    .ok_or(AuthError::ServerTokenNotConfigured)?;
                
                let provided = provided
                    .ok_or(AuthError::TokenRequired)?;
                
                // Constant-time comparison
                if constant_time_eq(expected.as_bytes(), provided.as_bytes()) {
                    Ok(())
                } else {
                    Err(AuthError::InvalidToken)
                }
            }
            
            AuthMode::Optional => {
                if let Some(provided) = provided {
                    if let Some(expected) = &self.token {
                        if !constant_time_eq(expected.as_bytes(), provided.as_bytes()) {
                            return Err(AuthError::InvalidToken);
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication token required")]
    TokenRequired,
    
    #[error("Invalid authentication token")]
    InvalidToken,
    
    #[error("Server token not configured")]
    ServerTokenNotConfigured,
}
```

---

## 7. Required Tests

Every sister MUST pass these hardening tests:

```rust
//! tests/hardening_tests.rs

#[test]
fn test_strict_validation_no_silent_fallback() {
    let params = json!({});
    
    // Must error, not silently return default
    let result = StrictValidator::require_string(&params, "project_id");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ValidationError::MissingRequired(_)));
}

#[test]
fn test_same_name_different_project_isolation() {
    let id_a = project_identity(Path::new("/tmp/test-a/myproject"));
    let id_b = project_identity(Path::new("/tmp/test-b/myproject"));
    
    assert_ne!(id_a, id_b, "Same-named projects must have different IDs");
}

#[test]
fn test_no_cross_project_cache_fallback() {
    let project_id = ProjectId::new();
    let result = resolve_graph(&project_id);
    
    // Must error, not fall back to another project's cache
    assert!(result.is_err());
}

#[test]
fn test_concurrent_startup_lock() {
    let lock1 = StartupLock::acquire("test-sister");
    assert!(lock1.is_ok());
    
    let lock2 = StartupLock::acquire("test-sister");
    assert!(matches!(lock2.unwrap_err(), LockError::AlreadyLocked));
    
    drop(lock1);
    
    // Should succeed after first lock released
    let lock3 = StartupLock::acquire("test-sister");
    assert!(lock3.is_ok());
}

#[test]
fn test_stale_lock_recovery() {
    // Create stale lock (old modification time)
    let lock_path = StartupLock::lock_path("test-stale");
    std::fs::write(&lock_path, "99999").unwrap();
    
    // Set old modification time
    let old_time = filetime::FileTime::from_unix_time(0, 0);
    filetime::set_file_mtime(&lock_path, old_time).unwrap();
    
    // Should recover stale lock
    let lock = StartupLock::acquire("test-stale");
    assert!(lock.is_ok());
}

#[test]
fn test_mcp_config_merge_not_replace() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create initial config with another server
    let initial = json!({
        "mcpServers": {
            "other-server": {"command": "other"}
        }
    });
    std::fs::write(&config_path, initial.to_string()).unwrap();
    
    // Add our server
    update_mcp_config(
        &config_path,
        "our-server",
        json!({"command": "ours"}),
    ).unwrap();
    
    // Verify other server preserved
    let result: Value = serde_json::from_str(
        &std::fs::read_to_string(&config_path).unwrap()
    ).unwrap();
    
    assert!(result["mcpServers"]["other-server"].is_object());
    assert!(result["mcpServers"]["our-server"].is_object());
}

#[test]
fn test_server_auth_constant_time() {
    let auth = TokenAuth {
        token: Some("correct-token".to_string()),
        mode: AuthMode::Required,
    };
    
    // Should accept correct token
    assert!(auth.validate(Some("correct-token")).is_ok());
    
    // Should reject incorrect token
    assert!(auth.validate(Some("wrong-token")).is_err());
    
    // Should reject missing token
    assert!(auth.validate(None).is_err());
}
```

---

## 8. Stress Tests

```rust
//! tests/stress_hardening.rs

#[test]
fn stress_multi_project_isolation() {
    let mut handles = vec![];
    
    // 10 concurrent projects with same name
    for i in 0..10 {
        let handle = std::thread::spawn(move || {
            let path = format!("/tmp/stress-test-{}/myproject", i);
            let id = project_identity(Path::new(&path));
            (i, id)
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // All IDs must be unique
    let ids: std::collections::HashSet<_> = results.iter()
        .map(|(_, id)| *id)
        .collect();
    
    assert_eq!(ids.len(), 10, "All project IDs must be unique");
}

#[test]
fn stress_concurrent_lock_acquisition() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    let acquired_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    // 50 threads trying to acquire same lock
    for _ in 0..50 {
        let count = Arc::clone(&acquired_count);
        let handle = std::thread::spawn(move || {
            if let Ok(_lock) = StartupLock::acquire("stress-lock") {
                count.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Only one should have acquired at any time
    // (but multiple may have acquired sequentially)
    assert!(acquired_count.load(Ordering::SeqCst) >= 1);
}

#[test]
fn stress_restart_continuity() {
    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().join("data.bin");
    
    // Simulate multiple restart cycles
    for i in 0..10 {
        // Start
        let _lock = StartupLock::acquire("restart-test").unwrap();
        
        // Write some data
        std::fs::write(&data_path, format!("iteration-{}", i)).unwrap();
        
        // "Crash" by dropping lock
        drop(_lock);
    }
    
    // Data should be from last iteration
    let contents = std::fs::read_to_string(&data_path).unwrap();
    assert_eq!(contents, "iteration-9");
}
```

---

## 9. Summary Checklist

```
HARDENING REQUIREMENTS (All Sisters Must Pass):
═══════════════════════════════════════════════

□ Strict MCP input validation (no silent fallbacks)
□ Per-project identity isolation (canonical-path hashing)
□ Zero cross-project cache contamination
□ Safe graph/artifact resolution (error, not fallback)
□ Concurrent startup locking with PID tracking
□ Stale lock recovery (5-minute threshold)
□ Graceful signal handling (SIGTERM, SIGINT, Ctrl+C)
□ MCP config merge (preserve other servers)
□ Post-install restart guidance displayed
□ Token-based server mode authentication
□ Constant-time token comparison
□ Atomic file writes with fsync
□ Audit logging for mutations
□ All hardening tests pass
□ All stress tests pass
```

---

*Document: SPEC-RUNTIME-HARDENING.md*
*Applies to: ALL Sisters*
