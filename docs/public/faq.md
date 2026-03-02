---
status: stable
---

# FAQ

## What is AgenticPlanning?

A persistent planning engine that gives AI agents living goals, crystallized decisions, and tracked commitments. It runs as an MCP server, CLI tool, Rust library, or FFI binding. Goals have physics (momentum, gravity) and feelings (urgency, confidence). Decisions preserve shadow paths. Commitments model entanglement between promises.

## What is an `.aplan` file?

A portable binary file containing all planning state — goals, decisions, commitments, dreams, federations, soul archives, and 24 precomputed indexes. It uses BLAKE3 checksums for integrity, atomic writes for crash safety, and Content-Length framed sections for fast access. Copy it to another machine and the full state travels with it.

## How is this different from a TODO list?

A TODO list is a flat checklist. AgenticPlanning goals have:
- **Physics** — Momentum (rate of progress), gravity (priority pull), inertia (resistance to change), energy (overall vitality)
- **Feelings** — Urgency, neglect, confidence, alignment, vitality — computed dynamically
- **Lifecycle** — Eight states from Draft through Reborn, with karma across reincarnation
- **Relationships** — Parent-child decomposition, typed links (Supports, Blocks, Enables, Conflicts)
- **Decisions** with shadow paths (preserved alternatives), causal chains, and reversibility scores
- **Commitments** with stakeholder weights, entanglement types, and breaking cost analysis

## Can multiple agents share planning state?

Yes, via the federation model. Agents join a federation around shared goals. Each agent owns their sub-goals and syncs state through the coordinator. Commitment entanglement analysis detects conflicts between agents' promises. Consensus protocol handles shared decisions.

## Does it work with Claude Code?

Yes. Add the MCP server to `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "agentic-planning": {
      "command": "aplan",
      "args": ["serve", "--file", "project.aplan"]
    }
  }
}
```

All 13 MCP tools become available to Claude. See [Playbooks](playbooks-agent-integration.md) for system prompt templates.

## How big do `.aplan` files get?

Roughly linear growth: ~2 KB per goal, ~1.5 KB per decision, ~1 KB per commitment.

| Project Scale | Approximate Size |
|--------------|-----------------|
| 10 goals | ~25 KB |
| 50 goals with decisions | ~150 KB |
| 100 goals, full state | ~300 KB |

See [Benchmarks](benchmarks.md) for detailed scaling data.

## Is it production-ready?

The core engine is stable and passes comprehensive tests including stress scenarios. The MCP server, CLI, and FFI are feature-complete. Bridge traits are defined with NoOp defaults — real implementations connecting to sister systems (agentic-memory, agentic-identity, etc.) are available separately.

## What happens if the process crashes mid-write?

Nothing is lost. Saves use atomic writes: data goes to `.aplan.tmp` first, then `fsync`, then rename to `.aplan`. If the process crashes, recovery detects the temp file on next load and promotes it. The `.aplan` file is always either the previous valid version or the new valid version — never a partial write.

## Can I use it from Python, JavaScript, or C?

Yes, all three:
- **Python** — Via FFI with `ctypes`. Load the shared library, call the 27 exported functions. See [Integration Guide](integration-guide.md).
- **JavaScript** — Via WASM. Install `@agentra/planning` from npm. Works in Node.js and browsers.
- **C** — Via the FFI crate directly. 27 `extern "C"` functions with `AplanResult` return codes.

## What are "goal feelings"?

Computed health indicators that surface a goal's subjective state:
- **Urgency** — Deadline pressure combined with priority. Critical goals near their deadline feel maximally urgent.
- **Neglect** — Time since last progress update. Rises automatically if the goal isn't touched.
- **Confidence** — Based on progress trajectory. Consistent forward movement builds confidence.
- **Alignment** — How well this goal fits with related goals in the singularity field.
- **Vitality** — Composite health score combining momentum, neglect, and confidence.

Feelings are computed on access, not stored. They always reflect the current moment.

## What is the "intention singularity"?

The convergence point where all goals, decisions, and commitments are viewed as a unified field. Singularity collapse produces:
- **Goal positions** — Where each goal sits based on priority, momentum, and alignment
- **Themes** — Natural clusters of related goals
- **Tension lines** — Conflicts between goals that need resolution
- **Golden path** — The highest-impact sequence of actions across all goals
- **Center** — The central intention that all goals orbit

Use it at the start of planning sessions to orient, or before high-impact decisions to understand alignment.

## How do I back up planning state?

Copy the `.aplan` file. It's self-contained — no external database, no service dependency, no configuration files. The format is version-safe across minor releases. The `version` field in the header ensures older files load correctly in newer engines.
