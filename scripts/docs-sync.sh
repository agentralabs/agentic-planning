#!/usr/bin/env bash
# Sync planning-docs source documentation into docs/public/ for distribution.
# Validates consistency between source and public artifacts.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PUBLIC_DIR="${SCRIPT_DIR}/docs/public"
MANIFEST="${PUBLIC_DIR}/sister.manifest.json"

green() { printf "\033[0;32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[0;33m%s\033[0m\n" "$*"; }
red() { printf "\033[0;31m%s\033[0m\n" "$*"; }

ERRORS=0

# ── Ensure public dir exists ─────────────────────────────────────────
mkdir -p "${PUBLIC_DIR}"

# ── Required public files ─────────────────────────────────────────────
REQUIRED_FILES=(
  "quickstart.md"
  "installation.md"
  "mcp-tools.md"
  "command-surface.md"
  "sister.manifest.json"
)

yellow "Checking required public docs..."
for file in "${REQUIRED_FILES[@]}"; do
  if [[ -f "${PUBLIC_DIR}/${file}" ]]; then
    green "  ✓ ${file}"
  else
    red "  ✗ ${file} MISSING"
    ERRORS=$((ERRORS + 1))
  fi
done

# ── Manifest validation ──────────────────────────────────────────────
if [[ -f "${MANIFEST}" ]]; then
  yellow "Validating manifest..."
  if command -v python3 &>/dev/null; then
    python3 -c "
import json, sys
with open('${MANIFEST}') as f:
    m = json.load(f)
    pages = m.get('pages', [])
    print(f'  Manifest has {len(pages)} pages')
    required = ['quickstart', 'installation', 'mcp-tools']
    ids = [p.get('id', '') for p in pages]
    for r in required:
        if r in ids:
            print(f'  ✓ page: {r}')
        else:
            print(f'  ✗ page: {r} MISSING')
            sys.exit(1)
" && green "  ✓ Manifest valid" || {
      red "  ✗ Manifest validation failed"
      ERRORS=$((ERRORS + 1))
    }
  else
    yellow "  (python3 not available, skipping manifest validation)"
  fi
fi

# ── Sync README sections into public docs ─────────────────────────────
README="${SCRIPT_DIR}/README.md"
if [[ -f "${README}" ]]; then
  yellow "Checking README consistency..."

  # Verify README mentions key sections
  for keyword in "Install" "Quick" "MCP" ".aplan"; do
    if grep -qi "${keyword}" "${README}"; then
      green "  ✓ README mentions '${keyword}'"
    else
      yellow "  ? README missing '${keyword}' section"
    fi
  done
fi

# ── planning-docs → public sync check ────────────────────────────────
PLANNING_DOCS="${SCRIPT_DIR}/planning-docs"
if [[ -d "${PLANNING_DOCS}" ]]; then
  yellow "Checking planning-docs coverage..."
  SPEC_COUNT=$(find "${PLANNING_DOCS}" -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')
  green "  ${SPEC_COUNT} spec files in planning-docs/"
fi

# ── Summary ───────────────────────────────────────────────────────────
echo ""
if [[ ${ERRORS} -eq 0 ]]; then
  green "Docs sync check passed."
else
  red "Docs sync check failed with ${ERRORS} error(s)."
  exit 1
fi
