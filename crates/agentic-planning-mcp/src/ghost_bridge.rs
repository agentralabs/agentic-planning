//! Ghost Writer Bridge — Syncs planning context to AI coding assistants.
//!
//! Detects Claude Code, Cursor, Windsurf, and Cody, then writes a
//! planning context summary (goals, decisions, commitments) to each
//! client's memory directory.
//!
//! Called from the stdio loop after each request (synchronous — no background thread).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use agentic_planning::PlanningEngine;

/// Cached client directories (detected once, reused on each sync).
pub struct GhostBridge {
    clients: Vec<ClientDir>,
    last_content_hash: u64,
}

struct ClientDir {
    name: &'static str,
    dir: PathBuf,
    filename: String,
}

impl GhostBridge {
    /// Create and detect all AI clients. Returns None if none found.
    pub fn new() -> Option<Self> {
        let clients = detect_all_memory_dirs();
        if clients.is_empty() {
            return None;
        }
        for c in &clients {
            eprintln!("[ghost_bridge] Planning context: {} at {:?}", c.name, c.dir);
        }
        Some(Self {
            clients,
            last_content_hash: 0,
        })
    }

    /// Sync current planning state to all detected clients.
    /// Only writes if content has changed (dedup via simple hash).
    pub fn sync(&mut self, engine: &PlanningEngine) {
        let markdown = build_planning_context(engine);

        let hash = simple_hash(&markdown);
        if hash == self.last_content_hash {
            return;
        }
        self.last_content_hash = hash;

        for client in &self.clients {
            let target = client.dir.join(&client.filename);
            if let Err(e) = atomic_write(&target, markdown.as_bytes()) {
                eprintln!("[ghost_bridge] Failed to sync to {:?}: {e}", target);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Context builder
// ═══════════════════════════════════════════════════════════════════

fn build_planning_context(engine: &PlanningEngine) -> String {
    let now = now_utc_string();

    let mut md = String::with_capacity(4096);
    md.push_str("# AgenticPlanning Context\n\n");
    md.push_str(&format!("> Auto-synced by Ghost Writer at {now}\n\n"));

    // Summary stats
    md.push_str("## Overview\n\n");
    md.push_str(&format!(
        "| Metric | Count |\n|--------|-------|\n| Goals | {} |\n| Decisions | {} |\n| Commitments | {} |\n\n",
        engine.goal_count(),
        engine.decision_count(),
        engine.commitment_count(),
    ));

    // Active goals
    let active_goals: Vec<_> = engine
        .goals()
        .filter(|g| matches!(g.status, agentic_planning::GoalStatus::Active))
        .collect();
    if !active_goals.is_empty() {
        md.push_str("## Active Goals\n\n");
        md.push_str("| Goal | Priority | Title |\n");
        md.push_str("|------|----------|-------|\n");
        for g in active_goals.iter().take(15) {
            let title = truncate(&g.title, 60);
            md.push_str(&format!(
                "| `{}` | {:?} | {title} |\n",
                short_id(&g.id.0),
                g.priority,
            ));
        }
        if active_goals.len() > 15 {
            md.push_str(&format!(
                "\n_...and {} more active goals_\n",
                active_goals.len() - 15
            ));
        }
        md.push('\n');
    }

    // Blocked goals
    let blocked: Vec<_> = engine
        .goals()
        .filter(|g| matches!(g.status, agentic_planning::GoalStatus::Blocked))
        .collect();
    if !blocked.is_empty() {
        md.push_str("## Blocked Goals\n\n");
        for g in blocked.iter().take(5) {
            md.push_str(&format!("- `{}` — {}\n", short_id(&g.id.0), truncate(&g.title, 80)));
        }
        md.push('\n');
    }

    // Pending decisions
    let pending_decisions: Vec<_> = engine
        .decisions()
        .filter(|d| matches!(d.status, agentic_planning::DecisionStatus::Pending | agentic_planning::DecisionStatus::Deliberating))
        .collect();
    if !pending_decisions.is_empty() {
        md.push_str("## Pending Decisions\n\n");
        for d in pending_decisions.iter().take(10) {
            let q = truncate(&d.question.question, 80);
            md.push_str(&format!("- `{}` [{:?}] — {q}\n", short_id(&d.id.0), d.status));
        }
        md.push('\n');
    }

    // Recent crystallized decisions
    let crystallized: Vec<_> = engine
        .decisions()
        .filter(|d| matches!(d.status, agentic_planning::DecisionStatus::Crystallized))
        .collect();
    if !crystallized.is_empty() {
        md.push_str("## Recent Decisions (Crystallized)\n\n");
        for d in crystallized.iter().take(5) {
            let q = truncate(&d.question.question, 60);
            let chosen = d
                .chosen
                .as_ref()
                .map(|p| truncate(&p.name, 40))
                .unwrap_or_else(|| "(none)".to_string());
            md.push_str(&format!("- {q} -> **{chosen}**\n"));
        }
        md.push('\n');
    }

    // Active commitments
    let active_commitments: Vec<_> = engine
        .commitments()
        .filter(|c| matches!(c.status, agentic_planning::CommitmentStatus::Active | agentic_planning::CommitmentStatus::AtRisk))
        .collect();
    if !active_commitments.is_empty() {
        md.push_str("## Active Commitments\n\n");
        md.push_str("| Commitment | To | Status | Weight |\n");
        md.push_str("|------------|-----|--------|--------|\n");
        for c in active_commitments.iter().take(10) {
            let desc = truncate(&c.promise.description, 40);
            md.push_str(&format!(
                "| {desc} | {} | {:?} | {:.1} |\n",
                truncate(&c.made_to.name, 20),
                c.status,
                c.weight,
            ));
        }
        md.push('\n');
    }

    md.push_str("---\n");
    md.push_str("_Auto-generated by AgenticPlanning. Do not edit manually._\n");
    md
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

fn short_id(uuid: &uuid::Uuid) -> String {
    let s = uuid.to_string();
    s[..8].to_string()
}

fn simple_hash(s: &str) -> u64 {
    // FNV-1a hash
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ═══════════════════════════════════════════════════════════════════
// Std-only UTC timestamp (avoids chrono dependency)
// ═══════════════════════════════════════════════════════════════════

fn now_utc_string() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let min = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let z = (secs / 86400) as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{min:02}:{s:02} UTC")
}

// ═══════════════════════════════════════════════════════════════════
// Multi-client detection
// ═══════════════════════════════════════════════════════════════════

fn detect_all_memory_dirs() -> Vec<ClientDir> {
    let home = match std::env::var("HOME").ok().map(PathBuf::from) {
        Some(h) => h,
        None => return vec![],
    };

    let candidates = [
        (
            "Claude Code",
            home.join(".claude").join("memory"),
            "PLANNING_CONTEXT.md",
        ),
        (
            "Cursor",
            home.join(".cursor").join("memory"),
            "agentic-planning.md",
        ),
        (
            "Windsurf",
            home.join(".windsurf").join("memory"),
            "agentic-planning.md",
        ),
        (
            "Cody",
            home.join(".sourcegraph").join("cody").join("memory"),
            "agentic-planning.md",
        ),
    ];

    let mut dirs = Vec::new();
    for (name, memory_dir, filename) in &candidates {
        if create_if_parent_exists(memory_dir) {
            dirs.push(ClientDir {
                name,
                dir: memory_dir.clone(),
                filename: filename.to_string(),
            });
        }
    }
    dirs
}

fn create_if_parent_exists(memory_dir: &Path) -> bool {
    if memory_dir.exists() {
        return true;
    }
    if let Some(parent) = memory_dir.parent() {
        if parent.exists() {
            return std::fs::create_dir_all(memory_dir).is_ok();
        }
    }
    false
}

fn atomic_write(target: &Path, content: &[u8]) -> Result<(), std::io::Error> {
    let tmp = target.with_extension("tmp");
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(content)?;
    f.sync_all()?;
    std::fs::rename(&tmp, target)?;
    Ok(())
}
