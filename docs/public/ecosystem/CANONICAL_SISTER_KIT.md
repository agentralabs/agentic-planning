# Canonical Sister Kit

## 1. Release Artifact Contract

Each sister ships source, tests, benchmark suite, and install scripts.

## 2. Install Contract Spec

Installers must support one-line install and deterministic binary selection.

## 3. Reusable CI Guardrails

Every sister must run format, clippy, tests, runtime checks, and docs checks.

## 4. README Canonical Layout

README must include hero pane, terminal pane, install, quickstart, and architecture.

## 5. MCP Canonical Profile

MCP server exposes consolidated tool facades and strict input validation.

## 6. Packaging Policy

Rust crates, Python, and npm bindings should remain version-aligned.

## 7. Versioning and Release Policy

Use semantic versioning and tagged release automation.

## 8. Design Asset Contract

Sisters publish hero, terminal, architecture, and benchmark SVG assets.

## 9. Env Var Namespace Contract

Runtime environment variables are prefixed and documented.

## 10. New-Sister Bootstrap

Start from canonical structure; wire docs, scripts, guardrails, and MCP.

## 11. Workspace Orchestrator Contract

Workspace dependencies must be explicit and reproducible.

## 12. Web Docs Grouping Contract

Public docs must map to stable page IDs via sister manifest.

## 13. Runtime Isolation and Universal MCP Hardening (Mandatory)

No silent fallback behavior for invalid enum/mode/depth/type parameters.
Deterministic per-project identity is required (canonical-path hashing or equivalent).
Do not bind to unrelated "latest cached" project state.
Stale/dead lock recovery is mandatory.
Support `desktop`, `terminal`, and `server` profiles.
Post-install output must include restart guidance and optional feedback guidance.
Server profile/runtime must enforce token-based auth gate (`AGENTIC_AUTH_TOKEN` or token file equivalent).
Release gate requires automated stress/regression proof for:
project isolation, lock contention, stale lock recovery, and MCP validation.

References:
- https://agentralabs.tech/docs/ecosystem-feature-reference
- https://agentralabs.tech/docs/sister-docs-catalog
- docs folder required for every sister
- web docs wiring is mandatory before release
- Installer strength/completeness is mandatory for every new sister.
- Before implementing a new sister installer, review `agentic-memory/scripts/install.sh`, `agentic-vision/scripts/install.sh`, and `agentic-codebase/scripts/install.sh` as benchmark baselines.
- `agentic-memory-mcp`, `agentic-vision-mcp`, `agentic-codebase-mcp`, and `agentic-identity-mcp` are treated as live ecosystem infrastructure.
- New sister planning, implementation, and validation must explicitly use those MCP servers where applicable (design support, integration checks, stress/regression checks).
