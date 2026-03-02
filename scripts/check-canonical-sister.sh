#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

assert_file() {
  [ -f "$1" ] || fail "Missing required file: $1"
}

assert_dir() {
  [ -d "$1" ] || fail "Missing required directory: $1"
}

assert_contains() {
  local pattern="$1"
  local target="$2"
  rg -nF "$pattern" "$target" >/dev/null || fail "Missing required pattern: ${pattern}"
}

assert_file "docs/ecosystem/CANONICAL_SISTER_KIT.md"
assert_file "docs/public/ecosystem/CANONICAL_SISTER_KIT.md"
assert_file "README.md"
assert_file "scripts/install.sh"
assert_file "scripts/check-install-commands.sh"
assert_file "scripts/check-runtime-hardening.sh"
assert_file "scripts/test-primary-problems.sh"
assert_file "scripts/check-docs-sync.sh"

assert_dir "assets"
assert_file "assets/github-hero-pane.svg"
assert_file "assets/github-terminal-pane.svg"
assert_file "assets/architecture-agentra.svg"
assert_file "assets/benchmark-chart.svg"

assert_file "docs/public/sister.manifest.json"
assert_contains '"key": "planning"' docs/public/sister.manifest.json
assert_contains '"name": "AgenticPlanning"' docs/public/sister.manifest.json

assert_contains '## Install' README.md
assert_contains '## Quickstart' README.md
assert_contains '## How It Works' README.md
assert_contains '<img src="assets/github-hero-pane.svg"' README.md
assert_contains '<img src="assets/github-terminal-pane.svg"' README.md
assert_contains 'href="#quickstart"' README.md
assert_contains 'href="#problems-solved"' README.md
assert_contains 'href="#how-it-works"' README.md
assert_contains 'href="#benchmarks"' README.md
assert_contains 'href="#install"' README.md

for doc in docs/public/quickstart.md docs/public/installation.md docs/public/concepts.md docs/public/integration-guide.md docs/public/api-reference.md docs/public/benchmarks.md docs/public/faq.md docs/public/file-format.md docs/public/experience-with-vs-without.md docs/public/command-surface.md docs/public/runtime-install-sync.md docs/public/playbooks-agent-integration.md; do
  assert_contains 'status: stable' "$doc"
done

assert_dir "crates/agentic-planning"
assert_dir "crates/agentic-planning-mcp"
assert_dir "crates/agentic-planning-cli"
assert_dir "crates/agentic-planning-ffi"
assert_dir "crates/agentic-planning-bridges"
assert_dir "python"
assert_file "python/pyproject.toml"
assert_dir "npm/wasm"
assert_file "npm/wasm/Cargo.toml"

assert_contains 'MAX_CONTENT_LENGTH_BYTES' crates/agentic-planning-mcp/src
assert_contains 'content-length:' crates/agentic-planning-mcp/src
assert_contains 'jsonrpc' crates/agentic-planning-mcp/src

assert_dir "paper"
PAPER_I_DIR="$(ls -d paper/paper-i-* 2>/dev/null | head -1)"
[ -n "$PAPER_I_DIR" ] || fail "Missing paper/paper-i-* directory"
ls "$PAPER_I_DIR"/*.tex >/dev/null 2>&1 || fail "Missing .tex file in $PAPER_I_DIR"
assert_file "$PAPER_I_DIR/references.bib"

assert_dir "crates/agentic-planning/benches"
assert_contains 'criterion_group!' crates/agentic-planning/benches
assert_contains 'bench_' crates/agentic-planning/benches

assert_dir "crates/agentic-planning/tests"
assert_contains 'scenario_15_large_graph' crates/agentic-planning/tests
assert_contains 'stress_' crates/agentic-planning/tests
assert_contains 'edge_case_' crates/agentic-planning/tests

echo "Canonical sister guardrails passed."
