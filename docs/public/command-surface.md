---
status: stable
---

# Command Surface

Cross-reference matrix showing which operations are available via MCP tools, CLI commands, FFI functions, and Rust API.

### Compact Facade Tools

AgenticPlanning uses operation-based routing for compact MCP tools. Each compact facade takes an `operation` plus optional `params`.

- `planning_goal`
- `planning_decision`
- `planning_commitment`
- `planning_progress`
- `planning_singularity`
- `planning_dream`
- `planning_counterfactual`
- `planning_chain`
- `planning_consensus`
- `planning_federate`
- `planning_metamorphosis`
- `planning_workspace`
- `planning_context_log`

Compact list mode: use `MCP_TOOL_SURFACE=compact`.

## Goal Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Create goal | `planning_goal` op=create | `goal create` | `aplan_goal_create` | `engine.create_goal()` |
| List goals | `planning_goal` op=list | `goal list` | `aplan_goal_list` | `engine.list_goals()` |
| Show goal | `planning_goal` op=show | `goal show` | `aplan_goal_get` | `engine.get_goal()` |
| Activate goal | `planning_goal` op=activate | `goal activate` | — | `engine.activate_goal()` |
| Update progress | `planning_goal` op=progress | `goal progress` | — | `engine.update_progress()` |
| Complete goal | `planning_goal` op=complete | `goal complete` | `aplan_goal_complete` | `engine.complete_goal()` |
| Abandon goal | `planning_goal` op=abandon | `goal abandon` | `aplan_goal_abandon` | `engine.abandon_goal()` |
| Pause goal | `planning_goal` op=pause | `goal pause` | `aplan_goal_pause` | `engine.pause_goal()` |
| Resume goal | `planning_goal` op=resume | `goal resume` | `aplan_goal_resume` | `engine.resume_goal()` |
| Block goal | `planning_goal` op=block | `goal block` | — | `engine.block_goal()` |
| Unblock goal | `planning_goal` op=unblock | `goal unblock` | — | `engine.unblock_goal()` |
| Decompose | `planning_goal` op=decompose | `goal decompose` | — | `engine.decompose_goal()` |
| Link goals | `planning_goal` op=link | `goal link` | — | `engine.link_goals()` |
| Goal tree | `planning_goal` op=tree | `goal tree` | — | `engine.goal_tree()` |
| Goal feelings | `planning_goal` op=feelings | `goal feelings` | — | `engine.goal_feelings()` |
| Goal physics | `planning_goal` op=physics | `goal physics` | — | `engine.goal_physics()` |
| Goal dream | `planning_goal` op=dream | `goal dream` | `aplan_dream_create` | `engine.dream_goal()` |
| Reincarnate | `planning_goal` op=reincarnate | `goal reincarnate` | — | `engine.reincarnate_goal()` |

## Decision Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Create | `planning_decision` op=create | `decision create` | `aplan_decision_create` | `engine.create_decision()` |
| Add option | `planning_decision` op=option | `decision option` | — | `engine.add_option()` |
| Crystallize | `planning_decision` op=crystallize | `decision crystallize` | `aplan_decision_crystallize` | `engine.crystallize()` |
| Show | `planning_decision` op=show | `decision show` | `aplan_decision_get` | `engine.get_decision()` |
| Shadows | `planning_decision` op=shadows | `decision shadows` | — | `engine.decision_shadows()` |
| Chain | `planning_decision` op=chain | `decision chain` | — | `engine.decision_chain()` |
| Archaeology | `planning_decision` op=archaeology | `decision archaeology` | — | `engine.decision_archaeology()` |
| Prophecy | `planning_decision` op=prophecy | `decision prophecy` | — | `engine.decision_prophecy()` |
| Regret | `planning_decision` op=regret | `decision regret` | — | `engine.decision_regret()` |
| Recrystallize | `planning_decision` op=recrystallize | `decision recrystallize` | — | `engine.recrystallize()` |
| List | — | — | `aplan_decision_list` | `engine.list_decisions()` |

## Commitment Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Create | `planning_commitment` op=create | `commitment create` | `aplan_commitment_create` | `engine.create_commitment()` |
| List | `planning_commitment` op=list | `commitment list` | `aplan_commitment_list` | `engine.list_commitments()` |
| Show | `planning_commitment` op=show | `commitment show` | `aplan_commitment_get` | `engine.get_commitment()` |
| Fulfill | `planning_commitment` op=fulfill | `commitment fulfill` | `aplan_commitment_fulfill` | `engine.fulfill_commitment()` |
| Break | `planning_commitment` op=break | `commitment break` | `aplan_commitment_break` | `engine.break_commitment()` |
| Renegotiate | `planning_commitment` op=renegotiate | `commitment renegotiate` | — | `engine.renegotiate()` |
| Entangle | `planning_commitment` op=entangle | `commitment entangle` | — | `engine.entangle()` |
| Inventory | `planning_commitment` op=inventory | `commitment inventory` | — | `engine.commitment_inventory()` |
| Due soon | `planning_commitment` op=due_soon | `commitment due-soon` | — | `engine.commitments_due_soon()` |
| At risk | `planning_commitment` op=at_risk | `commitment at-risk` | — | `engine.commitments_at_risk()` |

## Progress & Query Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Momentum | `planning_progress` op=momentum | `progress momentum` | — | `engine.momentum()` |
| Gravity | `planning_progress` op=gravity | `progress gravity` | — | `engine.gravity()` |
| Blockers | `planning_progress` op=blockers | `progress blockers` | `aplan_blockers_scan` | `engine.predict_blockers()` |
| Echoes | `planning_progress` op=echoes | `progress echoes` | `aplan_echoes_listen` | `engine.listen_echoes()` |
| Forecast | `planning_progress` op=forecast | `progress forecast` | — | `engine.forecast()` |
| Velocity | `planning_progress` op=velocity | `progress velocity` | — | `engine.velocity()` |
| Trend | `planning_progress` op=trend | `progress trend` | — | `engine.trend()` |

## Singularity Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Collapse | `planning_singularity` op=collapse | `singularity collapse` | `aplan_singularity_get` | `engine.collapse_singularity()` |
| Position | `planning_singularity` op=position | `singularity position` | — | `engine.position_goal()` |
| Path | `planning_singularity` op=path | `singularity path` | — | `engine.golden_path()` |
| Tensions | `planning_singularity` op=tensions | `singularity tensions` | — | `engine.tensions()` |
| Themes | `planning_singularity` op=themes | `singularity themes` | — | `engine.themes()` |
| Center | `planning_singularity` op=center | `singularity center` | — | `engine.intention_center()` |
| Vision | `planning_singularity` op=vision | `singularity vision` | — | `engine.unified_vision()` |

## Advanced Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Dream goal | `planning_dream` | `dream goal` | `aplan_dream_create` | `engine.dream_goal()` |
| Dream list | — | `dream history` | `aplan_dream_list` | `engine.list_dreams()` |
| Dream get | — | — | `aplan_dream_get` | `engine.get_dream()` |
| Counterfactual | `planning_counterfactual` | `counterfactual *` | — | `engine.counterfactual_*()` |
| Chain analysis | `planning_chain` | `chain *` | — | `engine.chain_*()` |
| Consensus | `planning_consensus` | `consensus *` | — | `engine.consensus_*()` |
| Federate | `planning_federate` | `federation *` | — | `engine.federation_*()` |
| Metamorphosis | `planning_metamorphosis` | `metamorphosis *` | — | `engine.metamorphosis_*()` |
| Workspace | `planning_workspace` | `workspace *` | — | `engine.workspace_*()` |
| Context log | `planning_context_log` | `context-log` | — | — |

## Utility Operations

| Operation | MCP Tool | CLI Command | FFI Function | Rust API |
|-----------|----------|-------------|--------------|----------|
| Status | resource `planning://status` | `status` | — | — |
| Version | — | `version` | `aplan_version` | — |
| Last error | — | — | `aplan_last_error` | — |
| Save state | — | — | `aplan_engine_save` | `engine.save()` |
| Load state | — | — | `aplan_engine_load` | `engine.load()` |
| Free string | — | — | `aplan_string_free` | — |
| Free engine | — | — | `aplan_engine_free` | (drop) |
| Serve (MCP) | — | `serve` | — | — |
| Daemon | — | `daemon *` | — | — |
