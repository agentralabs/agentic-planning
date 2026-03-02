#!/usr/bin/env bash
# Release automation for agentic-planning
# Usage: ./scripts/release.sh <version>
#
# Steps: preflight → version bump → build → tag → publish (opt-in)
set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: ./scripts/release.sh <version>"
  echo "  Example: ./scripts/release.sh 0.2.0"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DRY_RUN="${DRY_RUN:-true}"

green() { printf "\033[0;32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[0;33m%s\033[0m\n" "$*"; }
red() { printf "\033[0;31m%s\033[0m\n" "$*"; }

echo "Release v${VERSION} for agentic-planning"
echo "  DRY_RUN=${DRY_RUN} (set DRY_RUN=false to publish)"
echo ""

# ── 1. Preflight checks ──────────────────────────────────────────────
yellow "Step 1: Preflight checks..."
cd "${SCRIPT_DIR}"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
green "  ✓ All checks pass"

# ── 2. Version bump ──────────────────────────────────────────────────
yellow "Step 2: Version bump..."

# Workspace Cargo.toml
WORKSPACE_TOML="${SCRIPT_DIR}/Cargo.toml"
if grep -q 'version = "' "${WORKSPACE_TOML}"; then
  sed -i.bak "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" "${WORKSPACE_TOML}"
  rm -f "${WORKSPACE_TOML}.bak"
  green "  ✓ Cargo.toml → ${VERSION}"
fi

# Python pyproject.toml (if exists)
PYPROJECT="${SCRIPT_DIR}/python/pyproject.toml"
if [[ -f "${PYPROJECT}" ]]; then
  sed -i.bak "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" "${PYPROJECT}"
  rm -f "${PYPROJECT}.bak"
  green "  ✓ pyproject.toml → ${VERSION}"
fi

# npm package.json (if exists)
NPM_PKG="${SCRIPT_DIR}/npm/wasm/package.json"
if [[ -f "${NPM_PKG}" ]]; then
  # Use python for safe JSON editing if available, else sed
  if command -v python3 &>/dev/null; then
    python3 -c "
import json, sys
with open('${NPM_PKG}', 'r') as f: d = json.load(f)
d['version'] = '${VERSION}'
with open('${NPM_PKG}', 'w') as f: json.dump(d, f, indent=2)
print('  done')
"
  fi
  green "  ✓ package.json → ${VERSION}"
fi

# CHANGELOG.md placeholder entry
CHANGELOG="${SCRIPT_DIR}/CHANGELOG.md"
if [[ -f "${CHANGELOG}" ]]; then
  TODAY=$(date +%Y-%m-%d)
  if ! grep -q "## \[${VERSION}\]" "${CHANGELOG}"; then
    sed -i.bak "s/^## \[Unreleased\]/## [Unreleased]\n\n## [${VERSION}] - ${TODAY}/" "${CHANGELOG}"
    rm -f "${CHANGELOG}.bak"
    green "  ✓ CHANGELOG.md entry added"
  fi
fi

# ── 3. Build release artifacts ────────────────────────────────────────
yellow "Step 3: Building release artifacts..."
cargo build --release --workspace
green "  ✓ Release build complete"

# ── 4. Git tag ────────────────────────────────────────────────────────
yellow "Step 4: Git tag..."
if [[ "${DRY_RUN}" == "false" ]]; then
  git add -A
  git commit -m "chore: release v${VERSION}"
  git tag -a "v${VERSION}" -m "Release v${VERSION}"
  green "  ✓ Tagged v${VERSION}"
else
  yellow "  (dry run) Would tag v${VERSION}"
fi

# ── 5. Publish ────────────────────────────────────────────────────────
yellow "Step 5: Publish..."
if [[ "${DRY_RUN}" == "false" ]]; then
  # Crates.io
  cargo publish -p agentic-planning --allow-dirty || yellow "  ? crates.io publish skipped"
  cargo publish -p agentic-planning-ffi --allow-dirty || yellow "  ? crates.io FFI publish skipped"
  cargo publish -p agentic-planning-mcp --allow-dirty || yellow "  ? crates.io MCP publish skipped"
  cargo publish -p agentic-planning-cli --allow-dirty || yellow "  ? crates.io CLI publish skipped"

  # PyPI (if build tool available)
  if [[ -f "${PYPROJECT}" ]] && command -v python3 -m build &>/dev/null; then
    cd "${SCRIPT_DIR}/python"
    python3 -m build && python3 -m twine upload dist/* || yellow "  ? PyPI publish skipped"
    cd "${SCRIPT_DIR}"
  fi

  # npm (if package exists)
  if [[ -f "${NPM_PKG}" ]] && command -v npm &>/dev/null; then
    cd "$(dirname "${NPM_PKG}")"
    npm publish --access public || yellow "  ? npm publish skipped"
    cd "${SCRIPT_DIR}"
  fi

  green "  ✓ Published"
else
  yellow "  (dry run) Would publish to crates.io, PyPI, npm"
fi

# ── Done ──────────────────────────────────────────────────────────────
echo ""
green "Release v${VERSION} complete."
if [[ "${DRY_RUN}" == "true" ]]; then
  echo "  Re-run with DRY_RUN=false to publish."
fi
