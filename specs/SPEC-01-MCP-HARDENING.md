# SPEC-01: MCP Server Hardening

**Priority:** P0 — Server-crashing bugs
**File:** `agentic-planning-mcp/src/server.rs`
**Estimated changes:** ~200 lines

## Problem

28 `unwrap()` calls on parameter extraction can panic the MCP server on any
malformed request. Sister servers use `ok_or_else()` with proper MCP error
responses.

## Requirements

### R1: Replace all unwrap() with proper error handling

Every call to `get_string_param()`, `get_number_param()`, `get_bool_param()`
must use:

```rust
let value = get_string_param(params, "name")
    .ok_or_else(|| McpError::invalid_params("Missing required parameter: name"))?;
```

### R2: Implement prompts/get endpoint

Currently returns empty. Should return prompt content for the registered prompts.

### R3: Add missing resources

Add these resource URIs:
- `planning://dreams/{id}` — dream detail
- `planning://consensus/{id}` — federation consensus state
- `planning://workspace/{id}` — workspace summary

### R4: Input validation

Add validation for:
- String length limits (goal titles < 500 chars, descriptions < 10000 chars)
- Number bounds (weights 0.0..=1.0, importance 0.0..=1.0)
- Required fields presence check
- Invalid enum variant handling

## Acceptance Criteria

- [ ] Zero `unwrap()` calls remain on parameter extraction
- [ ] Malformed requests return proper MCP error, never panic
- [ ] `prompts/get` returns content for all registered prompts
- [ ] 3 new resource URIs resolve correctly
- [ ] All existing tests still pass
