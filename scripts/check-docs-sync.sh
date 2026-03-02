#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

for file in README.md docs/public/quickstart.md docs/public/installation.md docs/public/mcp-tools.md docs/public/sister.manifest.json; do
  [ -f "$file" ] || fail "Missing $file"
done

echo "Docs sync guardrails passed."
