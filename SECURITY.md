# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting Vulnerabilities

Please report security issues privately to `security@agentralabs.tech`.

Do not open public issues for security vulnerabilities. We aim to acknowledge reports within 48 hours and provide a fix or mitigation plan within 7 days for critical issues.

## Security Model

### File Integrity

- `.aplan` files use BLAKE3 checksums over all entity data
- Atomic writes via temp file + fsync + rename prevent partial writes
- Crash recovery detects orphaned `.aplan.tmp` files and promotes or discards them
- The integrity footer marker (`PLANEND\0`) validates file completeness

### MCP Server

- All tool inputs are validated before processing — no raw `unwrap()` on user input
- JSON-RPC 2.0 protocol errors return proper error codes (not panics)
- Unknown tools return `-32803` (TOOL_NOT_FOUND)
- Optional token-based authentication via `AGENTIC_AUTH_TOKEN` environment variable
- Server binds to stdio by default; HTTP mode requires explicit `--transport http`

### FFI Boundary

- All FFI functions accept and return C-compatible types
- Null pointer checks on every function entry
- UTF-8 validation on all string parameters
- Errors returned as `AplanResult` codes (never panics across FFI boundary)
- `aplan_last_error()` provides human-readable error details

### Input Validation

- Goal titles must be non-empty
- Priority values must be 1-5
- Progress percentages must be 0-100
- Entity IDs are validated UUIDs
- State transitions are enforced (cannot complete a Draft goal, cannot crystallize a Crystallized decision)

## Threat Model

### In Scope

- Data corruption via malformed `.aplan` files
- Denial of service via large or malicious MCP inputs
- Information leakage through error messages
- Unauthorized state modification via unprotected MCP server

### Out of Scope

- Network-level attacks (TLS is the transport layer's responsibility)
- Operating system compromise
- Physical access to the machine
- Attacks on sister systems connected via bridges (each sister handles its own security)
