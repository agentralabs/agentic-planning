#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

assert_contains() {
  local pattern="$1"
  local file="$2"
  rg -nF "$pattern" "$file" >/dev/null || fail "Missing pattern '${pattern}' in ${file}"
}

assert_contains 'curl -fsSL https://agentralabs.tech/install/planning' README.md
assert_contains 'npm install @agenticamem/planning' README.md
assert_contains 'pip install agentic-planning' README.md

assert_contains 'curl -fsSL https://agentralabs.tech/install/planning' docs/public/installation.md
assert_contains 'npm install @agenticamem/planning' docs/public/installation.md
assert_contains 'curl -fsSL https://agentralabs.tech/install/planning' docs/public/quickstart.md

echo "Install command guardrails passed."
