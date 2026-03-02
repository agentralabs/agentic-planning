# Documentation Gap Report — Phase 2

**Date:** 2026-03-01
**Scope:** 15 remaining stub documentation files in `docs/public/`
**Previous work:** SPEC-08 filled 8 core reference docs (mcp-tools, mcp-resources, mcp-prompts, cli-reference, command-surface, api-reference, ffi-reference, concepts)

---

## Executive Summary

After SPEC-08, the codebase has comprehensive **reference documentation** but zero **user-facing documentation**. 15 files remain as empty placeholders — no overview, no quickstart, no architecture explanation, no integration guide. A user encountering this project cannot understand what it does, how to install it, or how to use it.

**By category:**

| Category | Files | Current State | Impact |
|----------|-------|---------------|--------|
| Getting Started | 3 | overview (8 lines), quickstart (16 lines shell-only), installation (22 lines commands-only) | Users cannot understand what the project is or start using it |
| Architecture & Internals | 3 | architecture (8 lines), file-format (8 lines), configuration (8 lines) | Developers cannot understand how it works |
| Guides & Playbooks | 3 | integration-guide (8 lines), playbooks-agent-integration (8 lines), runtime-install-sync (8 lines) | No adoption path for agent builders |
| Narrative & Evidence | 3 | experience-with-vs-without (8 lines), initial-problem-coverage (12 lines), primary-problem-coverage (18 lines) | No motivation for adoption |
| Reference & Ops | 3 | troubleshooting (8 lines), benchmarks (8 lines), faq (8 lines) | No operational support |

**Total stub lines:** ~122 lines of placeholder text across 15 files
**Expected output:** ~3000 lines of substantive documentation

---

## File-by-File Gap Analysis

### Category 1: Getting Started

#### overview.md (8 lines → ~120 expected)
- **Current:** Title only — "AgenticPlanning documentation page: overview."
- **Missing:** What AgenticPlanning is, what problems it solves, core capabilities list, artifact description (.aplan files), sister ecosystem positioning, who should use it
- **Sister standard:** agentic-memory overview.md is ~50 lines with problem statement, capabilities, and cross-references

#### quickstart.md (16 lines → ~80 expected)
- **Current:** Install command + 3 bare shell commands with no explanation
- **Missing:** Prerequisites, step-by-step walkthrough with explanations, expected output, what to do next, MCP server setup
- **Sister standard:** agentic-memory quickstart.md is ~40 lines with narrative flow

#### installation.md (22 lines → ~80 expected)
- **Current:** 4 install commands (curl, cargo, npm, pip) with no context
- **Missing:** Prerequisites per platform, which binary to choose, verification steps, MCP server configuration for Claude Code / IDE, troubleshooting install issues
- **Sister standard:** Memory/vision installation docs explain each method with context

### Category 2: Architecture & Internals

#### architecture.md (8 lines → ~150 expected)
- **Current:** Title only
- **Missing:** Workspace layout (7 crates), per-crate responsibilities, data flow diagram, engine construction patterns, bridge architecture, persistence layer, MCP protocol handling
- **Sister standard:** agentic-memory architecture.md is ~120 lines with workspace tree and crate descriptions

#### file-format.md (8 lines → ~120 expected)
- **Current:** Title only
- **Missing:** .aplan binary format specification (magic number, header, sections, footer), checksum verification, crash recovery, atomic writes, version migration
- **Source:** file_format.rs has complete implementation to document

#### configuration.md (8 lines → ~80 expected)
- **Current:** Title only
- **Missing:** CLI global options (--file, --format, --verbose), MCP server configuration, engine modes (memory vs file-backed), bridge configuration, daemon settings, environment variables

### Category 3: Guides & Playbooks

#### integration-guide.md (8 lines → ~150 expected)
- **Current:** Title only
- **Missing:** Rust API integration, Python FFI binding, WASM/npm usage, MCP tool integration patterns, bridge trait implementation guide, sister integration (memory, identity, time, contract)
- **Sister standard:** Memory integration guide covers Python, WASM, MCP with code examples

#### playbooks-agent-integration.md (8 lines → ~120 expected)
- **Current:** Title only
- **Missing:** 30-second quick start, system prompt templates (minimal/standard/full), agent workflow examples (goal tracking, decision logging, commitment management), multi-agent federation setup
- **Sister standard:** Memory playbooks doc has 3-tier prompt templates and concrete agent examples

#### runtime-install-sync.md (8 lines → ~80 expected)
- **Current:** Title only
- **Missing:** Runtime detection, auto-install flow, MCP server sync, daemon lifecycle, workspace synchronization, multi-instance coordination

### Category 4: Narrative & Evidence

#### experience-with-vs-without.md (8 lines → ~120 expected)
- **Current:** Title only
- **Missing:** Before/after scenario simulation, concrete examples (goal drift without vs persistence with), numerical evidence (session continuity, decision recall), realistic limitations, long-horizon value proposition
- **Sister standard:** Memory experience doc has simulation with numbers, budget math, honest limitations

#### initial-problem-coverage.md (12 lines → ~80 expected)
- **Current:** 4 bullet points listing capability areas, no depth
- **Missing:** Detailed coverage per problem area, what's included in initial release, what's deferred, capability matrix with depth indicators

#### primary-problem-coverage.md (18 lines → ~100 expected)
- **Current:** 4 short paragraphs with one sentence each
- **Missing:** Deep explanation of each problem (goal drift, decision amnesia, commitment overload, progress blindness), how each is solved with specific tools/concepts, evidence of effectiveness

### Category 5: Reference & Ops

#### troubleshooting.md (8 lines → ~100 expected)
- **Current:** Title only
- **Missing:** Common errors and solutions, MCP connection issues, file format corruption recovery, CLI error codes, engine state debugging, performance issues

#### benchmarks.md (8 lines → ~80 expected)
- **Current:** Title only
- **Missing:** Operation latency (goal create, singularity collapse, file save/load), memory footprint, file size scaling, concurrent operation throughput, comparison with/without indexes
- **Source:** stress_tests.rs has performance-relevant test patterns

#### faq.md (8 lines → ~80 expected)
- **Current:** Title only
- **Missing:** Common questions (What is .aplan? How does it compare to TODO apps? Can multiple agents share? How big do files get? Is it production-ready?)

---

## Implementation Plan

| Spec | Category | Files | Priority | Est. Lines |
|------|----------|-------|----------|------------|
| SPEC-09 | Getting Started | overview.md, quickstart.md, installation.md | P1 | ~280 |
| SPEC-10 | Architecture & Internals | architecture.md, file-format.md, configuration.md | P1 | ~350 |
| SPEC-11 | Guides & Playbooks | integration-guide.md, playbooks-agent-integration.md, runtime-install-sync.md | P2 | ~350 |
| SPEC-12 | Narrative & Evidence | experience-with-vs-without.md, initial-problem-coverage.md, primary-problem-coverage.md | P2 | ~300 |
| SPEC-13 | Reference & Ops | troubleshooting.md, benchmarks.md, faq.md | P2 | ~260 |

**Total estimated:** ~1540 lines across 15 files in 5 specs
