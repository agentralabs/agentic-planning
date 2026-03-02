# AgenticPlanning

<div align="center">
  <a href="#quickstart">Quickstart</a> •
  <a href="#problems-solved">Problems Solved</a> •
  <a href="#how-it-works">How It Works</a> •
  <a href="#benchmarks">Benchmarks</a> •
  <a href="#install">Install</a>
</div>

<p align="center">
  <img src="assets/github-hero-pane.svg" alt="AgenticPlanning Hero" width="980" />
</p>

AgenticPlanning is persistent intention infrastructure for AI agents. It treats goals as living entities, decisions as crystallized forks with preserved shadows, commitments as weighted promises, and progress as measurable physics.

<p align="center">
  <img src="assets/github-terminal-pane.svg" alt="AgenticPlanning CLI" width="980" />
</p>

## Problems Solved

- Goal drift across sessions
- Decision loss without alternative-path memory
- Commitment overload and missed deadlines
- Missing progress momentum and blocker forecasting

## How It Works

- Core engine persists state in `.aplan` files with atomic writes.
- Query engine provides hierarchy, urgency, singularity, and prophecy views.
- MCP server exposes 12 consolidated planning tools for assistants.
- Bridges integrate with time, contract, memory, identity, and cognition systems.

<p align="center">
  <img src="assets/architecture-agentra.svg" alt="Architecture" width="980" />
</p>

## Benchmarks

<p align="center">
  <img src="assets/benchmark-chart.svg" alt="Benchmarks" width="980" />
</p>

The benchmark suite exercises create/list/singularity paths and includes Criterion reports in `crates/agentic-planning/benches`.

## Install

```bash
curl -fsSL https://agentralabs.tech/install/planning | bash
```

```bash
cargo install agentic-planning-cli
```

```bash
npm install @agenticamem/planning
```

```bash
pip install agentic-planning
```

## Quickstart

```bash
aplan goal create "Ship planning engine" --intention "Persistent intention infrastructure"
aplan goal list
aplan serve
```
