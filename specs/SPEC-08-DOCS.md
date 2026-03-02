# SPEC-08: Documentation

**Priority:** P3
**Files:** `docs/public/*.md` (8 files)
**Estimated changes:** ~2000 lines total

## Problem

All 8 core documentation files are empty placeholders containing only the title.

## Requirements

### R1: mcp-tools.md

Document every MCP tool with:
- Tool name
- Description (verb-first imperative per MCP Quality Standard)
- Required parameters with types
- Optional parameters with defaults
- Return format
- Example request/response

### R2: mcp-resources.md

Document every MCP resource:
- URI pattern
- Description
- Response format
- Example

### R3: mcp-prompts.md

Document every MCP prompt:
- Prompt name
- Arguments
- Template content

### R4: cli-reference.md

Document every CLI command:
- Syntax
- Arguments and flags
- Examples
- Output format variations

### R5: command-surface.md

Cross-reference matrix: which operations are available via MCP, CLI, FFI, API.

### R6: api-reference.md

Rust API documentation:
- PlanningEngine methods
- Type definitions
- Error handling
- Code examples

### R7: ffi-reference.md

FFI function signatures:
- C function declarations
- Parameter types
- Return values
- Error handling pattern
- Language binding examples

### R8: concepts.md

Core concepts:
- Goal physics model (momentum, gravity, inertia, energy)
- Goal feelings (urgency, neglect, confidence, alignment, vitality)
- Decision crystallization lifecycle
- Commitment entanglement types
- Dream system
- Federation model
- Metamorphosis and reincarnation
- Intention singularity

## Acceptance Criteria

- [ ] Each file has substantive content matching its topic
- [ ] Code examples compile
- [ ] Cross-references between docs are valid
