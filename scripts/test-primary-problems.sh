#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

assert_file() {
  [ -f "$1" ] || fail "Missing required file: $1"
}

assert_contains() {
  local pattern="$1"
  local file="$2"
  rg -nF "$pattern" "$file" >/dev/null || fail "Missing pattern '$pattern' in $file"
}

assert_file "docs/public/primary-problem-coverage.md"
assert_file "docs/public/initial-problem-coverage.md"

assert_contains "Goal Drift Across Sessions" docs/public/primary-problem-coverage.md
assert_contains "Decision Amnesia" docs/public/primary-problem-coverage.md
assert_contains "Commitment Overload" docs/public/primary-problem-coverage.md
assert_contains "Progress Blindness" docs/public/primary-problem-coverage.md

echo "Primary problem regression guardrails passed."
