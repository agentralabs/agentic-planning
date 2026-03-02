---
status: stable
---

# Integration Guide

AgenticPlanning exposes four integration surfaces: Rust API, Python FFI, WASM/npm, and MCP tools. Each provides full access to the planning engine.

## Rust API

Add the dependency:

```toml
[dependencies]
agentic-planning = { path = "crates/agentic-planning" }
```

Basic workflow — create a goal, track progress, complete it:

```rust
use agentic_planning::PlanningEngine;
use agentic_planning::types::{Priority, GoalStatus};

// In-memory engine (no persistence)
let mut engine = PlanningEngine::in_memory();

// Create a goal
let goal_id = engine.create_goal(
    "Ship v1.0",
    "Deliver the first stable release",
    Priority::High,
    None, // no deadline
    vec!["release".to_string()],
)?;

// Record progress
engine.record_progress(&goal_id, 0.3, "Core features done")?;
engine.record_progress(&goal_id, 0.7, "Tests passing")?;

// Check physics
let physics = engine.get_goal_physics(&goal_id)?;
println!("Momentum: {:.2}", physics.momentum);

// Complete
engine.complete_goal(&goal_id)?;
```

For file-backed persistence:

```rust
let mut engine = PlanningEngine::open("project.aplan")?;
// All mutations auto-save to disk
engine.create_goal("Ship v1.0", "Deliver", Priority::High, None, vec![])?;
// File is already saved — atomic write with BLAKE3 checksum
```

Error handling uses `PlanningError`:

```rust
use agentic_planning::error::PlanningError;

match engine.complete_goal(&goal_id) {
    Ok(()) => println!("Done"),
    Err(PlanningError::NotFound(id)) => eprintln!("No goal: {id}"),
    Err(PlanningError::InvalidTransition { from, to, .. }) =>
        eprintln!("Can't go from {from:?} to {to:?}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

## Python FFI

Load the shared library with `ctypes`:

```python
import ctypes
import json

lib = ctypes.CDLL("./target/release/libagentic_planning_ffi.dylib")

# Configure return types
lib.aplan_engine_new_memory.restype = ctypes.c_void_p
lib.aplan_goal_create.restype = ctypes.c_int
lib.aplan_goal_list.restype = ctypes.c_char_p
lib.aplan_string_free.argtypes = [ctypes.c_char_p]
lib.aplan_last_error.restype = ctypes.c_char_p

# Create engine
engine = lib.aplan_engine_new_memory()

# Create a goal
title = ctypes.c_char_p(b"Ship v1.0")
intention = ctypes.c_char_p(b"Deliver stable release")
result = lib.aplan_goal_create(engine, title, intention)
if result != 0:  # 0 = Ok
    error = lib.aplan_last_error()
    print(f"Error: {error.decode()}")

# List goals
json_ptr = lib.aplan_goal_list(engine)
goals = json.loads(json_ptr.decode())
lib.aplan_string_free(json_ptr)  # Always free returned strings

# Clean up
lib.aplan_engine_free(engine)
```

**AplanResult codes:** `Ok=0`, `NullPointer=1`, `InvalidUtf8=2`, `EngineError=3`, `NotFound=4`, `ValidationError=5`, `IoError=6`, `SerializationError=7`.

Always call `aplan_string_free` on every string pointer returned by the FFI. Failure to do so leaks memory.

## WASM / npm

```bash
npm install @agentra/planning
```

```javascript
import { PlanningEngine } from '@agentra/planning';

const engine = PlanningEngine.inMemory();
const goalId = engine.createGoal("Ship v1.0", "Deliver", "high");
engine.recordProgress(goalId, 0.5, "Halfway");
const physics = engine.getGoalPhysics(goalId);
console.log(`Momentum: ${physics.momentum}`);
```

Works in both Node.js and browser environments. Browser builds use the WASM binary bundled in the package.

## MCP Tool Integration

Configure AgenticPlanning as an MCP server in `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "agentic-planning": {
      "command": "aplan",
      "args": ["serve", "--file", "project.aplan"]
    }
  }
}
```

Tool call patterns:

```json
// Create a goal
{"tool": "planning_goal", "arguments": {"operation": "create", "title": "Ship v1.0", "intention": "Deliver stable release", "priority": "high"}}

// Record progress
{"tool": "planning_progress", "arguments": {"operation": "record", "goal_id": "abc123", "value": 0.5, "note": "Core done"}}

// Crystallize a decision
{"tool": "planning_decision", "arguments": {"operation": "crystallize", "decision_id": "def456", "chosen_path": "Option A", "reasoning": "Lower risk"}}

// Collapse intention singularity
{"tool": "planning_singularity", "arguments": {"operation": "collapse"}}
```

All 13 MCP tools: `planning_goal`, `planning_decision`, `planning_commitment`, `planning_progress`, `planning_singularity`, `planning_dream`, `planning_counterfactual`, `planning_chain`, `planning_consensus`, `planning_federate`, `planning_metamorphosis`, `planning_workspace`, `planning_context_log`.

## Bridge Implementation

To connect planning with a sister system, implement a bridge trait:

```rust
use agentic_planning_bridges::*;

struct MyMemoryBridge { /* connection to agentic-memory */ }

#[async_trait]
impl MemoryBridge for MyMemoryBridge {
    async fn persist_goal(&self, goal: &Goal) -> Result<MemoryId, BridgeError> {
        // Store goal as a memory node
        let id = self.memory_client.add(goal.title(), "fact").await?;
        Ok(MemoryId(id))
    }

    async fn get_goal_context(&self, goal_id: &GoalId) -> Result<Vec<MemoryRecord>, BridgeError> {
        // Retrieve related memories
        self.memory_client.similar(goal_id.to_string()).await
    }

    async fn persist_decision(&self, decision: &Decision) -> Result<MemoryId, BridgeError> {
        self.memory_client.add(decision.title(), "decision").await
    }

    async fn search_context(&self, query: &str) -> Result<Vec<MemoryRecord>, BridgeError> {
        self.memory_client.search(query).await
    }
}
```

Register bridges with the engine via `BridgeConfig`:

```rust
let config = BridgeConfig {
    memory: Some(Box::new(MyMemoryBridge::new())),
    identity: Some(Box::new(MyIdentityBridge::new())),
    ..Default::default()  // NoOp for all others
};
```

Nine bridge traits available: `TimeBridge`, `ContractBridge`, `MemoryBridge`, `IdentityBridge`, `CognitionBridge`, `VisionBridge`, `CodebaseBridge`, `CommBridge`. Each has a `NoOp` default that returns neutral values.

## File Persistence Patterns

**When to use file-backed mode:**
- Multi-session projects where goals must persist
- MCP server deployments with durable state
- Team collaboration via shared `.aplan` files

**When in-memory is sufficient:**
- Single-session ephemeral planning
- Tests and benchmarks
- WASM/browser environments
- Piped CLI workflows

**Workspace management:**

```bash
# Create a workspace scoped to a project
aplan workspace create --name "backend-v2"

# Switch between workspaces
aplan workspace switch --name "backend-v2"

# List all workspaces
aplan workspace list
```

The `.aplan` file is self-contained and portable. Copy it to another machine and the full planning state travels with it.
