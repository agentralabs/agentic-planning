# SPEC-05: CLI Completeness

**Priority:** P2
**File:** `agentic-planning-cli/src/main.rs`
**Estimated changes:** ~300 lines

## Problem

8 subcommands are stubs. Output formatting always uses JSON regardless of
`--format` flag. Several values are hardcoded.

## Requirements

### R1: Implement stub commands

| Command | Implementation |
|---------|---------------|
| `progress blockers` | Call `engine.scan_blocker_prophecy()`, display prophecies |
| `progress echoes` | Call `engine.listen_progress_echoes()`, display echoes |
| `singularity collapse` | Call `engine.get_intention_singularity()`, display full result |
| `singularity path` | Call `engine.get_intention_singularity()`, display golden_path |
| `singularity tensions` | Call `engine.get_intention_singularity()`, display tensions |
| `singularity themes` | Call `engine.get_intention_singularity()`, display themes |
| `singularity vision` | Call `engine.synthesize_vision()`, display vision |
| `federation consensus` | Call `engine.get_federation_consensus()`, display result |

### R2: Fix output formatting

`emit_output()` must respect the `OutputFormat`:
- `Json`: Current behavior (serde_json::to_string_pretty)
- `Text`: Human-readable key=value pairs, one per line
- `Table`: Aligned columns with headers (use simple padding, no crate needed)

### R3: Fix hardcoded values

- Stakeholder role: accept `--role` CLI arg (default "stakeholder")
- Stakeholder importance: accept `--importance` CLI arg (default 0.6)
- Entanglement strength: accept `--strength` CLI arg (default 0.8)

### R4: Serve mode persistence

In stdio serve mode, persist engine mutations to disk after each write operation.
Call `engine.save()` after any tool that modifies state.

## Acceptance Criteria

- [ ] All 8 stub commands produce real output
- [ ] `--format text` and `--format table` work correctly
- [ ] Hardcoded values replaced with CLI args + defaults
- [ ] Serve mode persists state changes
