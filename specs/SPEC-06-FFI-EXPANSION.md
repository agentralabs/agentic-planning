# SPEC-06: FFI Expansion

**Priority:** P2
**File:** `agentic-planning-ffi/src/lib.rs`
**Estimated changes:** ~500 lines

## Problem

Only 4 FFI functions exist. Agentic-memory FFI has 30+. Python/Node bindings
cannot access decisions, commitments, dreams, federations, or any query
operations.

## Requirements

### R1: Error code enum

```rust
#[repr(C)]
pub enum AplanResult {
    Ok = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    EngineError = 3,
    NotFound = 4,
    ValidationError = 5,
    IoError = 6,
    SerializationError = 7,
}
```

All functions return `AplanResult` instead of null pointers.

### R2: Engine lifecycle

- `aplan_engine_new(path) -> AplanResult` (existing, update return type)
- `aplan_engine_new_memory() -> AplanResult` (in-memory mode)
- `aplan_engine_free(handle)` (existing)
- `aplan_engine_save(handle) -> AplanResult`
- `aplan_engine_load(handle, path) -> AplanResult`

### R3: Goal operations

- `aplan_goal_create(handle, json) -> *mut c_char` (existing, keep)
- `aplan_goal_get(handle, id) -> *mut c_char`
- `aplan_goal_list(handle) -> *mut c_char`
- `aplan_goal_pause(handle, id, reason) -> AplanResult`
- `aplan_goal_resume(handle, id) -> AplanResult`
- `aplan_goal_abandon(handle, id) -> AplanResult`
- `aplan_goal_complete(handle, id) -> AplanResult`

### R4: Decision operations

- `aplan_decision_create(handle, json) -> *mut c_char`
- `aplan_decision_get(handle, id) -> *mut c_char`
- `aplan_decision_list(handle) -> *mut c_char`
- `aplan_decision_crystallize(handle, id, chosen) -> AplanResult`

### R5: Commitment operations

- `aplan_commitment_create(handle, json) -> *mut c_char`
- `aplan_commitment_get(handle, id) -> *mut c_char`
- `aplan_commitment_list(handle) -> *mut c_char`
- `aplan_commitment_fulfill(handle, id) -> AplanResult`
- `aplan_commitment_break(handle, id, reason) -> AplanResult`

### R6: Dream operations

- `aplan_dream_create(handle, json) -> *mut c_char`
- `aplan_dream_get(handle, id) -> *mut c_char`
- `aplan_dream_list(handle) -> *mut c_char`

### R7: Query operations

- `aplan_singularity_get(handle) -> *mut c_char`
- `aplan_blockers_scan(handle) -> *mut c_char`
- `aplan_echoes_listen(handle) -> *mut c_char`

### R8: Utility

- `aplan_string_free(ptr)` (existing)
- `aplan_version() -> *const c_char`
- `aplan_last_error() -> *const c_char` (thread-local error message)

## Acceptance Criteria

- [ ] 25+ FFI functions exposed
- [ ] All functions return `AplanResult` or JSON string
- [ ] Thread-local error message accessible via `aplan_last_error()`
- [ ] Basic smoke tests for each function
- [ ] C header file generated (cbindgen)
