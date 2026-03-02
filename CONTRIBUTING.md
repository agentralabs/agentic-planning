# Contributing to AgenticPlanning

## Development flow
1. Branch from `main`.
2. Implement changes with tests.
3. Run local checks:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `make guardrails`
4. Open a PR with scope, risk, and verification details.

## Project layout
- `crates/agentic-planning`: core engine.
- `crates/agentic-planning-mcp`: MCP server.
- `crates/agentic-planning-cli`: CLI surface (`aplan`).
- `crates/agentic-planning-ffi`: C ABI bridge.
- `crates/agentic-planning-bridges`: sister integration traits.

## Coding expectations
- Keep file format compatibility stable.
- Preserve strict MCP validation behavior.
- Add scenario/stress coverage for behavior changes.
