---
status: stable
---

# Quickstart

Get from zero to a working planning session in under 10 minutes.

## Prerequisites

- Rust toolchain (1.75+) or a pre-built binary
- Optional: Claude Code for MCP integration

## Step 1: Install

```bash
cargo install agentic-planning-cli
```

Verify installation:

```bash
aplan version
# AgenticPlanning v0.1.0
```

## Step 2: Create a Goal

```bash
aplan goal create "Build authentication system" \
  --intention "Secure user access for the platform" \
  --priority high \
  --activate

# Output:
# Goal created: a1b2c3d4-...
# Status: Active
# Priority: High
# Physics: momentum=0.00 gravity=0.80 inertia=0.35 energy=1.00
```

The `--activate` flag moves the goal straight to Active status, skipping Draft. Physics are calculated from priority and emotional weight.

## Step 3: Track Progress

```bash
aplan goal progress a1b2c3d4 30 --note "Database schema designed"

# Output:
# Progress: 30%
# Velocity: 0.30
# Momentum: 0.18
```

Each progress update records a timestamped point. Velocity and momentum are recalculated from the progress history — they reflect how fast work is moving and whether it's accelerating or decelerating.

## Step 4: Check Feelings

```bash
aplan goal feelings a1b2c3d4

# Output:
# Urgency:    0.65  (deadline pressure)
# Neglect:    0.08  (time since attention)
# Confidence: 0.71  (success likelihood)
# Alignment:  0.50  (singularity fit)
# Vitality:   0.68  (overall health)
```

Feelings are computed indicators, not stored values. Rising neglect or falling confidence are early warning signals that a goal needs attention.

## Step 5: Make a Decision

```bash
aplan decision create \
  --question "Which auth provider to use?" \
  --context "Need OAuth2, good docs, reasonable pricing"

# Decision created: e5f6a7b8-...
# Status: Pending
```

Add options and crystallize:

```bash
aplan decision add-path e5f6a7b8 \
  --name "Auth0" \
  --pros "Mature,Good docs" \
  --cons "Expensive at scale"

aplan decision add-path e5f6a7b8 \
  --name "Supabase Auth" \
  --pros "Integrated,Free tier" \
  --cons "Younger ecosystem"

aplan decision crystallize e5f6a7b8 --chosen "Auth0"

# Decision crystallized.
# Shadow path preserved: "Supabase Auth"
```

The unchosen path becomes a **shadow** — preserved for future reference or recrystallization if context changes.

## Step 6: Start the MCP Server

```bash
aplan serve
```

This starts the MCP server over stdio, making all 13 planning tools available to Claude Code. Configure it in your Claude Code settings:

```json
{
  "mcpServers": {
    "agentic-planning": {
      "command": "aplan",
      "args": ["serve"]
    }
  }
}
```

## Next Steps

- [CLI Reference](cli-reference.md) — Full command documentation
- [MCP Tools](mcp-tools.md) — All 13 MCP tools with parameters
- [Concepts](concepts.md) — Goal physics, decision crystallization, commitment entanglement
- [Integration Guide](integration-guide.md) — Rust API, Python FFI, WASM patterns
