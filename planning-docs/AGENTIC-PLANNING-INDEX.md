# AGENTIC PLANNING — COMPLETE SPECIFICATION INDEX

> **Sister:** #8 of 25
> **Format:** .aplan
> **Status:** Ready for Claude Code Implementation
> **Dependencies:** Time ✅ + Contract ✅

---

## Executive Summary

AgenticPlanning provides **persistent intention infrastructure** for AI agents. Goals are living entities that persist across sessions, decisions crystallize reality with preserved alternatives, commitments have physical weight, and progress follows physics (momentum, gravity, blockers).

**22 Impossible Inventions:**
- 6 Goal Consciousness inventions (Living Goals, Intention Singularity, Goal Dreaming, Relationships, Reincarnation, Metamorphosis)
- 6 Decision Crystallization inventions (Crystallization, Counterfactual, Chains, Archaeology, Prophecy, Consensus)
- 4 Progress Physics inventions (Momentum, Gravity, Blocker Prophecy, Progress Echo)
- 4 Commitment Physics inventions (Weight, Entanglement, Fulfillment Energy, Renegotiation)
- 2 Collective Planning inventions (Goal Federation, Collective Dreaming)

**12 MCP Tools** with ~144 operations total.

---

## Document Index

### AgenticPlanning-Specific Specs

| Document | Size | Contents |
|----------|------|----------|
| `AGENTIC-PLANNING-22-INVENTIONS.md` | 71KB | All 22 inventions with data structures and MCP tools |
| `AGENTIC-PLANNING-SPEC-PART1.md` | 46KB | Overview, Core Concepts, Data Structures, File Format |
| `AGENTIC-PLANNING-SPEC-PART2.md` | 65KB | Write Engine, Query Engine, Indexes, Validation |
| `AGENTIC-PLANNING-SPEC-PART3.md` | 76KB | CLI, MCP Server, Sister Integration, 16 Test Scenarios |
| `AGENTIC-PLANNING-SPEC-PART4.md` | 60KB | Performance, Security/Hardening, Research Paper, Implementation |
| `SPEC-PROJECT-STRUCTURE-PLANNING.md` | 15KB | Repository layout, Cargo.toml, module organization |

**Total AgenticPlanning specs: ~333KB**

### Universal Sister Specs (Apply to ALL Sisters)

| Document | Size | Contents |
|----------|------|----------|
| `SPEC-INSTALLER-UNIVERSAL.md` | 19KB | Profile-based installer (desktop\|terminal\|server), merge-only MCP config |
| `SPEC-RUNTIME-HARDENING.md` | 22KB | Strict validation, project isolation, concurrent locking, auth |
| `SPEC-RELEASE-PUBLISH.md` | 19KB | Version strategy, release gates, multi-registry publishing |
| `SPEC-DOCS-PUBLIC-SYNC.md` | 14KB | Documentation sync workflow, quality checks |
| `SPEC-CI-GUARDRAILS.md` | 19KB | 50 guardrail sections, automated enforcement |

**Total universal specs: ~93KB**

---

## Implementation Checklist

### Phase 1: Core (Ship First)
```
□ Living Goals (lifecycle, status, hierarchy)
□ Decision Crystallization (basic)
□ Commitment Weight (basic)
□ Progress Momentum (basic)
□ .aplan file format (read/write)
□ 4 MCP tools (planning_goal, planning_decision, planning_commitment, planning_progress)
□ 16 test scenarios passing
```

### Phase 2: Essential
```
□ Intention Singularity
□ Goal Dreaming (simple)
□ Goal Relationships
□ Counterfactual Projection
□ Progress Gravity
□ CLI commands
□ Daemon support
```

### Phase 3: Advanced
```
□ Goal Reincarnation
□ Goal Metamorphosis
□ Decision Chains
□ Decision Archaeology
□ Blocker Prophecy
□ Progress Echo
```

### Phase 4: Collective
```
□ Goal Federation
□ Collective Dreaming
□ Full MCP tool suite (12 tools)
```

---

## MCP Tool Summary

| Tool | Operations | Priority |
|------|------------|----------|
| `planning_goal` | create, list, show, activate, progress, complete, abandon, pause, resume, block, unblock, decompose, link, tree, feelings, physics, dream, reincarnate | P1 |
| `planning_decision` | create, option, crystallize, show, shadows, chain, archaeology, prophecy, regret, recrystallize | P1 |
| `planning_commitment` | create, list, show, fulfill, break, renegotiate, entangle, inventory, due_soon, at_risk | P1 |
| `planning_progress` | momentum, gravity, blockers, echoes, forecast, velocity, trend | P1 |
| `planning_singularity` | collapse, position, path, tensions, themes, center, vision | P2 |
| `planning_dream` | goal, collective, interpret, insights, accuracy, history | P2 |
| `planning_counterfactual` | project, compare, learn, timeline | P3 |
| `planning_chain` | trace, cascade, roots, leaves, visualize | P3 |
| `planning_consensus` | start, round, synthesize, vote, status | P4 |
| `planning_federate` | create, join, sync, handoff, status, members | P4 |
| `planning_metamorphosis` | detect, approve, history, predict, stage | P3 |
| `planning_workspace` | create, switch, list, compare, merge, delete | P2 |

---

## Sister Bridge Dependencies

| Sister | Required | Integration |
|--------|----------|-------------|
| Temporal Bridge | ✅ Yes | Deadlines, scheduling, urgency calculation |
| AgenticContract | ✅ Yes | Commitment enforcement, policy boundaries |
| AgenticMemory | Optional | Goal persistence as memories |
| AgenticIdentity | Optional | Decision/commitment signing |
| AgenticCognition | Optional | User modeling for prioritization |

---

## Hardening Requirements (Mandatory)

From `SPEC-RUNTIME-HARDENING.md`:

```
✅ Strict MCP input validation (no silent fallbacks)
✅ Per-project identity isolation (canonical-path hashing)
✅ Zero cross-project cache contamination
✅ Concurrent startup locking with stale-lock recovery
✅ Merge-only MCP client config updates
✅ Profile-based universal installer (desktop|terminal|server)
✅ Post-install restart guidance
✅ Token-based server mode authentication
✅ Atomic file operations
✅ Audit logging
```

---

## CI/CD Requirements

From `SPEC-CI-GUARDRAILS.md`:

```
50 Guardrail Sections:
  §1-10:  Code Quality (fmt, clippy, build)
  §11-20: Test Coverage (unit, integration, stress)
  §21-30: Hardening Verification
  §31-40: Documentation
  §41-47: Sister-Specific Docs
  §48:    Contract Compliance
  §49:    Release Gates
  §50:    Final Verification
```

---

## File Format Summary

```
.aplan File Structure:
├── HEADER (128 bytes)
│   ├── Magic: "PLAN"
│   ├── Version, flags, timestamps
│   ├── Entity counts
│   ├── Section offsets
│   └── Blake3 checksum
├── GOAL GRAPH SECTION
├── DECISION CRYSTALS SECTION
├── COMMITMENT LEDGER SECTION
├── DREAM ARCHIVE SECTION
├── FEDERATION STATE SECTION
├── INDEXES SECTION
└── FOOTER (64 bytes)
```

---

## Claude Code Instructions

```markdown
## Implementation Order

1. Read ALL specs before coding
2. Create project structure per SPEC-PROJECT-STRUCTURE-PLANNING.md
3. Implement data structures from SPEC-PART1
4. Implement file format from SPEC-PART1
5. Implement write engine from SPEC-PART2
6. Implement query engine from SPEC-PART2
7. Implement CLI from SPEC-PART3
8. Implement MCP server from SPEC-PART3
9. Run all 16 test scenarios from SPEC-PART3
10. Apply hardening from SPEC-RUNTIME-HARDENING.md
11. Verify CI guardrails from SPEC-CI-GUARDRAILS.md

## Success Criteria

- cargo test passes (all 16 scenarios)
- cargo clippy passes (no warnings)
- cargo fmt passes
- MCP tool count = 12
- Hardening tests pass
- Installer verification passes
```

---

## Package Distribution

| Registry | Package | Version |
|----------|---------|---------|
| crates.io | `agentic-planning` | 0.1.0 |
| crates.io | `agentic-planning-mcp` | 0.1.0 |
| crates.io | `agentic-planning-bridges` | 0.1.0 |
| PyPI | `agentic-planning` | 0.1.0 |
| npm | `@agentic/planning` | 0.1.0 |

---

## Summary Stats

```
AgenticPlanning Specification:
─────────────────────────────
Documents:         11 files
Total size:        ~426KB
Inventions:        22
MCP tools:         12
Operations:        ~144
Test scenarios:    16
Guardrail sections: 50
Dependencies:      2 required (Time, Contract)
```

---

*Sister #8 of 25: AgenticPlanning*
*The sister that makes agents finish what they start.*
*Goals that live. Decisions that crystallize. Progress that has physics.*
