#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

assert_contains() {
  local pattern="$1"
  local target="$2"
  rg -nF "$pattern" "$target" >/dev/null || fail "Missing pattern '$pattern' in $target"
}

MCP_SRC="crates/agentic-planning-mcp/src"

assert_contains "MAX_CONTENT_LENGTH_BYTES" "$MCP_SRC"
assert_contains "content-length:" "$MCP_SRC"
assert_contains "jsonrpc" "$MCP_SRC"
assert_contains "AGENTIC_AUTH_TOKEN" "$MCP_SRC"

echo "Runtime hardening guardrails passed."
