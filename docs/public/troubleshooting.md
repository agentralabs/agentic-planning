---
status: stable
---

# Troubleshooting

Common issues with diagnosis and resolution.

## MCP Connection Issues

**Server won't start:**
- Verify the binary is installed: `which aplan` or `aplan version`
- Check file permissions on the binary: `ls -la $(which aplan)`
- Ensure `settings.json` has the correct path in `mcpServers.agentic-planning.command`
- If using `--file`, verify the directory exists and is writable

**Tool calls fail:**
- Verify JSON-RPC 2.0 format: requests must include `jsonrpc`, `method`, `params`, and `id`
- Check parameter names match the tool schema (e.g., `operation`, `title`, `intention`)
- Confirm the tool name is one of the 13 registered tools (see [MCP Reference](mcp-reference.md))

**Server crashes:**
- Check stderr output for panic messages
- Common cause: invalid parameter types (e.g., passing a string where a number is expected)
- If file-backed: verify the `.aplan` file isn't corrupted (see File Format Issues below)

## File Format Issues

**Corrupted `.aplan` file (checksum mismatch on load):**
- The BLAKE3 checksum over entity data doesn't match the stored checksum
- If a `.aplan.tmp` file exists alongside the `.aplan` file, the engine automatically uses the `.aplan` file and deletes the stale `.tmp`
- If only `.aplan.tmp` exists (crash during rename), the engine recovers by promoting it to `.aplan`
- If the file is truly corrupted (no valid backup): delete the file and start fresh

**File won't load:**
- **Version mismatch** — The `version` field doesn't match the expected version (currently `1`). Update the `aplan` binary.
- **Missing magic number** — File doesn't start with `PLAN` (0x504C414E). The file may not be an `.aplan` file or is severely corrupted.
- **Truncated write** — File is shorter than expected. The write was interrupted before completion. Check for a `.aplan.tmp` backup.
- **Integrity marker missing** — Footer doesn't contain `PLANEND\0`. File is truncated or corrupted.

**Permission errors:**
- Verify read/write permissions on the `.aplan` file and its parent directory
- Check available disk space — atomic writes need room for both the `.aplan` file and a temporary `.aplan.tmp`

## CLI Errors

**Unknown command:**
- Run `aplan --help` for the full command list
- Check your `aplan` version matches the expected feature set: `aplan version`

**Entity not found:**
- Verify the entity ID is correct (UUIDs are 32 hex characters)
- Confirm you're using the right `--file` flag pointing to the correct `.aplan` file
- Without `--file`, each CLI invocation uses an empty in-memory engine — previous entities won't exist

**Invalid state transition:**
- Goals follow a strict lifecycle: Draft → Active → Blocked → Paused → Completed → Abandoned → Superseded → Reborn
- You cannot complete a Draft goal (activate it first)
- You cannot crystallize an already-Crystallized decision (regret it first, then recrystallize)
- You cannot fulfill a Broken commitment (it's terminal)
- Run `aplan goal show <id>` to check current status before attempting a transition

## Engine State Issues

**Goals stuck in wrong state:**
- Valid transitions are enforced by the engine. If a goal appears stuck, check its blocker list: `aplan goal show <id>`
- Blocked goals must be explicitly unblocked before they can progress
- Paused goals must be resumed before they can be completed

**Indexes out of sync:**
- This should not happen during normal operation (indexes update incrementally on every mutation)
- If suspected: reload the engine from file. Index rebuild happens automatically on load.
- In-memory mode: indexes are always in sync since there's no persistence layer to diverge from

**Stale feelings/physics:**
- Goal feelings (urgency, neglect, confidence, alignment, vitality) are computed on access, not stored
- They reflect the current time relative to the last activity — this is by design
- Rising neglect simply means the goal hasn't received a progress update recently

## Performance Issues

**Slow singularity collapse:**
- Singularity collapse is O(n) where n is the number of active goals
- For projects with 50+ goals, expect ~10-15ms
- Use goal filtering (by status, priority, or tag) to reduce the working set before collapsing

**Large file sizes:**
- Growth is roughly linear: ~2 KB per goal, ~1.5 KB per decision, ~1 KB per commitment
- Consider archiving completed goals if the file exceeds ~500 KB
- Soul archives for reincarnated goals add additional size

**Memory usage:**
- The in-memory engine holds all entities and indexes in RAM
- A 100-goal project uses ~500 KB of memory
- For larger datasets, use file-backed mode — entity data is still held in memory, but persistence is managed

## Error Code Reference

FFI functions return `AplanResult` codes:

| Code | Name | Meaning |
|------|------|---------|
| 0 | `Ok` | Operation succeeded |
| 1 | `NullPointer` | A null pointer was passed to a function |
| 2 | `InvalidUtf8` | A C string contained invalid UTF-8 |
| 3 | `EngineError` | Internal engine error (see `aplan_last_error()` for details) |
| 4 | `NotFound` | The requested entity doesn't exist |
| 5 | `ValidationError` | Input validation failed (empty title, invalid priority, etc.) |
| 6 | `IoError` | File I/O error (permission denied, disk full, etc.) |
| 7 | `SerializationError` | JSON serialization/deserialization failed |

Call `aplan_last_error()` after any non-zero result for a human-readable error message.
