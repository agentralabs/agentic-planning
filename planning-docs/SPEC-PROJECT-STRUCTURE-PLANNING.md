# AGENTIC PLANNING — SPEC-PROJECT-STRUCTURE

> **Sister:** #8 of 25 (AgenticPlanning)
> **Format:** .aplan
> **Pattern:** Canonical Sister Structure

---

## Repository Layout

```
agentic-planning/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                    # Main CI pipeline
│   │   ├── release.yml               # Release automation
│   │   ├── docs-sync.yml             # Documentation sync
│   │   └── security.yml              # Security scanning
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── sister_integration.md
│   └── PULL_REQUEST_TEMPLATE.md
│
├── crates/
│   ├── agentic-planning/             # Core library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── goal.rs
│   │   │   │   ├── decision.rs
│   │   │   │   ├── commitment.rs
│   │   │   │   ├── dream.rs
│   │   │   │   ├── federation.rs
│   │   │   │   └── physics.rs
│   │   │   ├── engine/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── write.rs
│   │   │   │   ├── query.rs
│   │   │   │   └── physics.rs
│   │   │   ├── storage/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── file.rs
│   │   │   │   ├── atomic.rs
│   │   │   │   ├── header.rs
│   │   │   │   └── indexes.rs
│   │   │   ├── inventions/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── living_goals.rs
│   │   │   │   ├── intention_singularity.rs
│   │   │   │   ├── goal_dreaming.rs
│   │   │   │   ├── decision_crystallization.rs
│   │   │   │   ├── blocker_prophecy.rs
│   │   │   │   ├── progress_echo.rs
│   │   │   │   ├── commitment_physics.rs
│   │   │   │   ├── goal_reincarnation.rs
│   │   │   │   └── collective_dreaming.rs
│   │   │   ├── validation/
│   │   │   │   ├── mod.rs
│   │   │   │   └── strict.rs
│   │   │   └── error.rs
│   │   ├── tests/
│   │   │   ├── goal_tests.rs
│   │   │   ├── decision_tests.rs
│   │   │   ├── commitment_tests.rs
│   │   │   ├── physics_tests.rs
│   │   │   ├── storage_tests.rs
│   │   │   └── integration_tests.rs
│   │   └── benches/
│   │       └── planning_bench.rs
│   │
│   ├── agentic-planning-mcp/         # MCP server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── server.rs
│   │   │   ├── tools/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── goal.rs
│   │   │   │   ├── decision.rs
│   │   │   │   ├── commitment.rs
│   │   │   │   ├── progress.rs
│   │   │   │   ├── singularity.rs
│   │   │   │   ├── dream.rs
│   │   │   │   ├── counterfactual.rs
│   │   │   │   ├── chain.rs
│   │   │   │   ├── consensus.rs
│   │   │   │   ├── federate.rs
│   │   │   │   ├── metamorphosis.rs
│   │   │   │   └── workspace.rs
│   │   │   ├── resources.rs
│   │   │   ├── prompts.rs
│   │   │   ├── auth.rs
│   │   │   ├── daemon/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── process.rs
│   │   │   │   ├── daemonize.rs
│   │   │   │   ├── launchd.rs
│   │   │   │   ├── systemd.rs
│   │   │   │   └── windows.rs
│   │   │   ├── cli/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── goal.rs
│   │   │   │   ├── decision.rs
│   │   │   │   ├── commitment.rs
│   │   │   │   └── progress.rs
│   │   │   └── audit/
│   │   │       ├── mod.rs
│   │   │       └── log.rs
│   │   ├── tests/
│   │   │   ├── phase01_goal_lifecycle.rs
│   │   │   ├── phase02_decision_crystallization.rs
│   │   │   ├── phase03_commitment_physics.rs
│   │   │   ├── phase04_progress_momentum.rs
│   │   │   ├── phase05_intention_singularity.rs
│   │   │   ├── phase06_goal_dreaming.rs
│   │   │   ├── phase07_blocker_prophecy.rs
│   │   │   ├── phase08_reincarnation.rs
│   │   │   ├── phase09_federation.rs
│   │   │   ├── phase10_mcp_tools.rs
│   │   │   ├── phase11_daemon_cli.rs
│   │   │   ├── phase12_edge_stress.rs
│   │   │   └── mcp_tool_count.rs
│   │   └── README.md
│   │
│   └── agentic-planning-bridges/     # Sister integrations
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── time.rs
│       │   ├── contract.rs
│       │   ├── memory.rs
│       │   ├── identity.rs
│       │   ├── cognition.rs
│       │   └── hydra.rs
│       └── tests/
│           └── bridge_tests.rs
│
├── bindings/
│   ├── python/
│   │   ├── pyproject.toml
│   │   ├── src/
│   │   │   └── agentic_planning/
│   │   │       ├── __init__.py
│   │   │       └── _core.pyi
│   │   └── tests/
│   │       └── test_planning.py
│   │
│   ├── node/
│   │   ├── package.json
│   │   ├── src/
│   │   │   ├── index.ts
│   │   │   └── types.ts
│   │   └── tests/
│   │       └── planning.test.ts
│   │
│   └── wasm/
│       ├── Cargo.toml
│       ├── src/
│       │   └── lib.rs
│       └── pkg/
│           └── .gitkeep
│
├── scripts/
│   ├── install.sh                    # Universal installer
│   ├── install.ps1                   # Windows installer
│   ├── check-install-commands.sh     # Verify installer
│   ├── release.sh                    # Release automation
│   ├── benchmark.sh                  # Run benchmarks
│   └── docs-sync.sh                  # Sync docs to public
│
├── docs/
│   ├── README.md
│   ├── QUICKSTART.md
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── CLI.md
│   ├── MCP-TOOLS.md
│   ├── INVENTIONS.md
│   ├── SISTER-INTEGRATION.md
│   └── RESEARCH-PAPER.md
│
├── specs/
│   ├── SPEC-OVERVIEW.md
│   ├── SPEC-DATA-STRUCTURES.md
│   ├── SPEC-FILE-FORMAT.md
│   ├── SPEC-WRITE-ENGINE.md
│   ├── SPEC-QUERY-ENGINE.md
│   ├── SPEC-INDEXES.md
│   ├── SPEC-CLI.md
│   ├── SPEC-MCP.md
│   ├── SPEC-BRIDGES.md
│   ├── SPEC-TESTS.md
│   ├── SPEC-PERFORMANCE.md
│   ├── SPEC-SECURITY.md
│   └── SPEC-RESEARCH-PAPER.md
│
├── examples/
│   ├── basic_goal.rs
│   ├── decision_making.rs
│   ├── commitment_tracking.rs
│   ├── intention_singularity.rs
│   └── claude_desktop_config.json
│
├── Cargo.toml                        # Workspace root
├── Cargo.lock
├── rust-toolchain.toml
├── .gitignore
├── .rustfmt.toml
├── clippy.toml
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── SECURITY.md
```

---

## Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/agentic-planning",
    "crates/agentic-planning-mcp",
    "crates/agentic-planning-bridges",
    "bindings/wasm",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/agentralabs/agentic-planning"
homepage = "https://agentic.so/planning"
documentation = "https://docs.agentic.so/planning"
keywords = ["ai", "agent", "planning", "goals", "mcp"]
categories = ["development-tools", "command-line-utilities"]
rust-version = "1.75"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Hashing
blake3 = "1.5"

# UUID
uuid = { version = "1.0", features = ["v4", "serde"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Async
tokio = { version = "1.0", features = ["full"] }

# Graph
petgraph = "0.6"

# CLI
clap = { version = "4.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# File watching
notify = "6.0"

# MCP
mcp-rs = { version = "0.1", optional = true }

# Testing
tempfile = "3.0"
criterion = "0.5"

# Sister dependencies
temporal-bridge = { version = "0.1", optional = true }
agentic-contract = { version = "0.1", optional = true }
agentic-memory = { version = "0.4", optional = true }
agentic-identity = { version = "0.3", optional = true }

[profile.release]
lto = true
codegen-units = 1
strip = true
```

---

## Core Crate Cargo.toml

```toml
[package]
name = "agentic-planning"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Persistent intention infrastructure for AI agents"
readme = "../../README.md"

[features]
default = []
full = ["time-bridge", "contract-bridge", "memory-bridge", "identity-bridge"]
time-bridge = ["temporal-bridge"]
contract-bridge = ["agentic-contract"]
memory-bridge = ["agentic-memory"]
identity-bridge = ["agentic-identity"]

[dependencies]
serde.workspace = true
serde_json.workspace = true
bincode.workspace = true
blake3.workspace = true
uuid.workspace = true
chrono.workspace = true
tokio.workspace = true
petgraph.workspace = true
tracing.workspace = true
thiserror.workspace = true

# Optional sister bridges
temporal-bridge = { workspace = true, optional = true }
agentic-contract = { workspace = true, optional = true }
agentic-memory = { workspace = true, optional = true }
agentic-identity = { workspace = true, optional = true }

[dev-dependencies]
tempfile.workspace = true
criterion.workspace = true
tokio = { workspace = true, features = ["test-util"] }

[[bench]]
name = "planning_bench"
harness = false
```

---

## MCP Crate Cargo.toml

```toml
[package]
name = "agentic-planning-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "MCP server for AgenticPlanning"
readme = "README.md"

[[bin]]
name = "agentic-planning"
path = "src/main.rs"

[features]
default = []
daemon = ["notify"]

[dependencies]
agentic-planning = { path = "../agentic-planning" }
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
tokio.workspace = true
clap.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
thiserror.workspace = true
anyhow.workspace = true

# Daemon support
notify = { workspace = true, optional = true }

[dev-dependencies]
tempfile.workspace = true
tokio = { workspace = true, features = ["test-util"] }
```

---

## Module Organization

### Core Library (agentic-planning)

```rust
//! src/lib.rs

pub mod types;
pub mod engine;
pub mod storage;
pub mod inventions;
pub mod validation;
pub mod error;

// Re-exports
pub use types::*;
pub use engine::PlanningEngine;
pub use error::{Error, Result};

// Feature-gated bridges
#[cfg(feature = "time-bridge")]
pub mod bridges;
```

### MCP Server (agentic-planning-mcp)

```rust
//! src/main.rs

mod server;
mod tools;
mod resources;
mod prompts;
mod auth;
mod cli;
mod audit;

#[cfg(feature = "daemon")]
mod daemon;

use clap::Parser;

#[derive(Parser)]
#[command(name = "agentic-planning")]
#[command(bin_name = "aplan")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Goal commands
    Goal(cli::GoalCommand),
    
    // Decision commands
    Decision(cli::DecisionCommand),
    
    // Commitment commands
    Commitment(cli::CommitmentCommand),
    
    // Progress commands
    Progress(cli::ProgressCommand),
    
    // Singularity
    Singularity(cli::SingularityCommand),
    
    // Status
    Status,
    
    // MCP server
    Serve {
        #[arg(long, default_value = "stdio")]
        mode: String,
        
        #[arg(long, default_value = "3000")]
        port: u16,
    },
    
    // Daemon
    #[cfg(feature = "daemon")]
    Daemon(daemon::DaemonCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve { mode, port } => {
            server::run(&mode, port).await
        }
        Commands::Status => {
            cli::status().await
        }
        // ... other commands
        _ => Ok(()),
    }
}
```

---

## File Naming Conventions

```
NAMING RULES:
═════════════

Source files:
  - snake_case.rs
  - One module per file when small
  - Directory with mod.rs when complex

Test files:
  - phase{NN}_{description}.rs (for MCP)
  - {module}_tests.rs (for unit tests)
  - integration_tests.rs (for integration)

Spec files:
  - SPEC-{NAME}.md (SCREAMING-KEBAB)

Documentation:
  - {NAME}.md (SCREAMING or Title case)

Scripts:
  - {name}.sh or {name}.ps1 (kebab-case)

Binary names:
  - agentic-planning (kebab-case)
  - aplan (short alias)
```

---

## Critical Files

```
MUST EXIST (CI will fail without):
══════════════════════════════════

Repository root:
  ✓ README.md
  ✓ LICENSE-MIT
  ✓ LICENSE-APACHE
  ✓ CHANGELOG.md
  ✓ SECURITY.md
  ✓ Cargo.toml (workspace)
  ✓ rust-toolchain.toml

Each crate:
  ✓ Cargo.toml
  ✓ src/lib.rs or src/main.rs
  ✓ README.md (for publishable crates)

Scripts:
  ✓ scripts/install.sh
  ✓ scripts/check-install-commands.sh

CI:
  ✓ .github/workflows/ci.yml
  ✓ .github/workflows/release.yml
```

---

*Document: SPEC-PROJECT-STRUCTURE.md*
