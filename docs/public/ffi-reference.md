---
status: stable
---

# FFI Reference

C-compatible FFI exported from `agentic-planning-ffi`. All functions use `extern "C"` linkage.

## Error Handling

```c
typedef enum {
    APLAN_OK                 = 0,  // Success
    APLAN_NULL_POINTER       = 1,  // Null pointer argument
    APLAN_INVALID_UTF8       = 2,  // UTF-8 conversion error
    APLAN_ENGINE_ERROR       = 3,  // Planning engine error
    APLAN_NOT_FOUND          = 4,  // Resource not found
    APLAN_VALIDATION_ERROR   = 5,  // Validation or state transition error
    APLAN_IO_ERROR           = 6,  // File I/O error
    APLAN_SERIALIZATION_ERROR = 7  // JSON serialization error
} AplanResult;
```

Use `aplan_last_error()` to retrieve a detailed error message after any non-OK result. Error messages are stored in thread-local storage.

## Memory Management

Functions returning `char*` allocate heap memory. **Always** free returned strings with `aplan_string_free()`.

```c
char* json = aplan_goal_list(handle);
// ... use json ...
aplan_string_free(json);
```

---

## Engine Lifecycle

### aplan_engine_new

```c
AplanHandle* aplan_engine_new(void);
```

Create a new in-memory planning engine. Returns an opaque handle.

### aplan_engine_new_memory

```c
AplanHandle* aplan_engine_new_memory(void);
```

Alias for `aplan_engine_new`.

### aplan_engine_new_file

```c
AplanHandle* aplan_engine_new_file(const char* path);
```

Create a file-backed planning engine. Loads existing state if the file exists. Returns `NULL` on error.

### aplan_engine_free

```c
void aplan_engine_free(AplanHandle* handle);
```

Free an engine handle. Null-safe (no-op if `NULL`).

### aplan_engine_save

```c
AplanResult aplan_engine_save(AplanHandle* handle);
```

Persist engine state to disk. Only meaningful for file-backed engines.

### aplan_engine_load

```c
AplanResult aplan_engine_load(AplanHandle* handle, const char* path);
```

Load state from a file into the engine, replacing current state.

---

## Goal Operations

### aplan_goal_create

```c
char* aplan_goal_create(AplanHandle* handle, const char* title, const char* intention);
```

Create a new goal. Returns JSON representation of the created goal. Caller must free with `aplan_string_free`.

### aplan_goal_get

```c
char* aplan_goal_get(AplanHandle* handle, const char* id);
```

Get goal by UUID string. Returns JSON or `NULL` if not found.

### aplan_goal_list

```c
char* aplan_goal_list(AplanHandle* handle);
```

List all goals. Returns JSON array.

### aplan_goal_pause

```c
AplanResult aplan_goal_pause(AplanHandle* handle, const char* id, const char* reason);
```

Pause a goal. `reason` may be `NULL`.

### aplan_goal_resume

```c
AplanResult aplan_goal_resume(AplanHandle* handle, const char* id);
```

Resume a paused goal.

### aplan_goal_abandon

```c
AplanResult aplan_goal_abandon(AplanHandle* handle, const char* id);
```

Abandon a goal.

### aplan_goal_complete

```c
AplanResult aplan_goal_complete(AplanHandle* handle, const char* id);
```

Mark a goal as complete.

---

## Decision Operations

### aplan_decision_create

```c
char* aplan_decision_create(AplanHandle* handle, const char* json);
```

Create a decision from JSON input. Minimum: `{"question": "..."}`. Returns JSON of created decision.

### aplan_decision_get

```c
char* aplan_decision_get(AplanHandle* handle, const char* id);
```

Get decision by UUID. Returns JSON or `NULL`.

### aplan_decision_list

```c
char* aplan_decision_list(AplanHandle* handle);
```

List all decisions. Returns JSON array.

### aplan_decision_crystallize

```c
AplanResult aplan_decision_crystallize(AplanHandle* handle, const char* id, const char* chosen_path_id);
```

Crystallize a decision by selecting a path UUID.

---

## Commitment Operations

### aplan_commitment_create

```c
char* aplan_commitment_create(AplanHandle* handle, const char* json);
```

Create commitment from JSON. Requires `promise` and `stakeholder` fields. Returns JSON.

### aplan_commitment_get

```c
char* aplan_commitment_get(AplanHandle* handle, const char* id);
```

Get commitment by UUID. Returns JSON or `NULL`.

### aplan_commitment_list

```c
char* aplan_commitment_list(AplanHandle* handle);
```

List all commitments. Returns JSON array.

### aplan_commitment_fulfill

```c
AplanResult aplan_commitment_fulfill(AplanHandle* handle, const char* id);
```

Mark commitment as fulfilled.

### aplan_commitment_break

```c
AplanResult aplan_commitment_break(AplanHandle* handle, const char* id, const char* reason);
```

Break a commitment with a reason.

---

## Dream Operations

### aplan_dream_create

```c
char* aplan_dream_create(AplanHandle* handle, const char* goal_id);
```

Generate a dream for the given goal. Returns JSON.

### aplan_dream_get

```c
char* aplan_dream_get(AplanHandle* handle, const char* id);
```

Get dream by UUID. Returns JSON or `NULL`.

### aplan_dream_list

```c
char* aplan_dream_list(AplanHandle* handle);
```

List all dreams. Returns JSON array.

---

## Query Operations

### aplan_singularity_get

```c
char* aplan_singularity_get(AplanHandle* handle);
```

Compute and return the intention singularity. Returns JSON.

### aplan_blockers_scan

```c
char* aplan_blockers_scan(AplanHandle* handle);
```

Scan for predicted blockers. Returns JSON array of `BlockerProphecy`.

### aplan_echoes_listen

```c
char* aplan_echoes_listen(AplanHandle* handle);
```

Listen for progress echoes. Returns JSON array of `ProgressEcho`.

---

## Utility Functions

### aplan_string_free

```c
void aplan_string_free(char* s);
```

Free a string returned by any FFI function. Null-safe.

### aplan_version

```c
const char* aplan_version(void);
```

Return the library version as a static string. Do **not** free this pointer.

### aplan_last_error

```c
const char* aplan_last_error(void);
```

Return the last error message (thread-local). Returns `NULL` if no error. Do **not** free this pointer.

---

## Language Binding Example (Python)

```python
import ctypes

lib = ctypes.CDLL("libagentic_planning_ffi.so")

# Configure return types
lib.aplan_engine_new.restype = ctypes.c_void_p
lib.aplan_goal_create.restype = ctypes.c_char_p
lib.aplan_goal_list.restype = ctypes.c_char_p

# Create engine
handle = lib.aplan_engine_new()

# Create a goal
json_result = lib.aplan_goal_create(
    handle,
    b"Ship v2.0",
    b"Deliver next release"
)
print(json_result.decode("utf-8"))

# Clean up
lib.aplan_string_free(json_result)
lib.aplan_engine_free(handle)
```

## Language Binding Example (C)

```c
#include <stdio.h>

// Link with -lagentic_planning_ffi
extern AplanHandle* aplan_engine_new(void);
extern char* aplan_goal_create(AplanHandle*, const char*, const char*);
extern void aplan_string_free(char*);
extern void aplan_engine_free(AplanHandle*);

int main() {
    AplanHandle* engine = aplan_engine_new();

    char* goal = aplan_goal_create(engine, "Ship v2.0", "Next release");
    if (goal) {
        printf("Created: %s\n", goal);
        aplan_string_free(goal);
    }

    aplan_engine_free(engine);
    return 0;
}
```
