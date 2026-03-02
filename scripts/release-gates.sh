#!/usr/bin/env bash
# ============================================================================
# Release gates for agentic-planning
# Validates all pre-publication requirements without publishing.
#
# Run before `release.sh`:
#   bash scripts/release-gates.sh
#   bash scripts/release-gates.sh --version 0.2.0
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

VERSION="${1:-}"
if [[ -n "$VERSION" && "$VERSION" == "--version" ]]; then
  VERSION="${2:-}"
  shift 2 || true
elif [[ -n "$VERSION" && "$VERSION" != --* ]]; then
  shift || true
fi

green()  { printf "\033[0;32m  ✓ %s\033[0m\n" "$*"; }
yellow() { printf "\033[0;33m  ? %s\033[0m\n" "$*"; }
red()    { printf "\033[0;31m  ✗ %s\033[0m\n" "$*"; }
bold()   { printf "\033[1m%s\033[0m\n" "$*"; }
section(){ printf "\n\033[1;36mGate %s: %s\033[0m\n" "$1" "$2"; }

ERRORS=0
WARNINGS=0
err()  { red "$*"; ERRORS=$((ERRORS + 1)); }
warn() { yellow "$*"; WARNINGS=$((WARNINGS + 1)); }

# Resolve version from Cargo.toml if not provided
if [[ -z "$VERSION" ]]; then
  VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2 2>/dev/null || echo "")
fi

echo ""
bold "╔══════════════════════════════════════════════════════════╗"
bold "║       agentic-planning release gates                    ║"
bold "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Version: ${VERSION:-unknown}"
echo ""

# ── Gate 1: Version consistency ─────────────────────────────────────────
section "1" "Version consistency"

CARGO_VER=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2 2>/dev/null || echo "")
if [[ -n "$CARGO_VER" ]]; then
  green "Cargo.toml: $CARGO_VER"
else
  err "Cannot read version from Cargo.toml"
fi

# Python version
PY_VER=""
if [[ -f python/pyproject.toml ]]; then
  PY_VER=$(grep '^version' python/pyproject.toml | head -1 | cut -d'"' -f2 2>/dev/null || echo "")
  if [[ "$PY_VER" == "$CARGO_VER" ]]; then
    green "pyproject.toml: $PY_VER (matches)"
  else
    err "pyproject.toml: $PY_VER (expected $CARGO_VER)"
  fi
fi

# npm version
NPM_VER=""
if [[ -f npm/wasm/pkg/package.json ]]; then
  NPM_VER=$(grep '"version"' npm/wasm/pkg/package.json | head -1 | sed 's/.*"\([0-9][^"]*\)".*/\1/' 2>/dev/null || echo "")
  if [[ "$NPM_VER" == "$CARGO_VER" ]]; then
    green "package.json: $NPM_VER (matches)"
  else
    err "package.json: $NPM_VER (expected $CARGO_VER)"
  fi
fi

# npm package name
if [[ -f npm/wasm/pkg/package.json ]]; then
  NPM_NAME=$(grep '"name"' npm/wasm/pkg/package.json | head -1 | sed 's/.*"\(@[^"]*\)".*/\1/' 2>/dev/null || echo "")
  if [[ "$NPM_NAME" == "@agenticamem/planning" ]]; then
    green "npm package name: $NPM_NAME"
  else
    err "npm package name: $NPM_NAME (expected @agenticamem/planning)"
  fi
fi

# ── Gate 2: Build verification ──────────────────────────────────────────
section "2" "Build verification"

if cargo build --release --workspace 2>/dev/null; then
  green "release build succeeds"
else
  err "release build failed"
fi

# Check binary existence
for bin in agentic-planning-mcp aplan; do
  if [[ -f "target/release/$bin" ]]; then
    green "binary: $bin"
  else
    err "binary missing: $bin"
  fi
done

# ── Gate 3: Full test suite ─────────────────────────────────────────────
section "3" "Full test suite"

TEST_OUTPUT=$(cargo test --workspace 2>&1)
TEST_RESULT=$?
TOTAL_TESTS=$(echo "$TEST_OUTPUT" | grep "test result:" | awk '{sum += $4} END {print sum+0}')
FAILED_TESTS=$(echo "$TEST_OUTPUT" | grep "test result:" | awk '{sum += $6} END {print sum+0}')

if [[ $TEST_RESULT -eq 0 ]]; then
  green "$TOTAL_TESTS tests pass, 0 failures"
else
  err "$FAILED_TESTS test failures out of $TOTAL_TESTS"
fi

# Stress test count
STRESS_COUNT=$(echo "$TEST_OUTPUT" | grep -c "stress_" || true)
if [[ $STRESS_COUNT -ge 10 ]]; then
  green "$STRESS_COUNT stress tests"
else
  warn "only $STRESS_COUNT stress tests (expected ≥10)"
fi

# MCP test count
MCP_COUNT=$(echo "$TEST_OUTPUT" | grep -c "mcp_\|mcp::" || true)
if [[ $MCP_COUNT -ge 5 ]]; then
  green "$MCP_COUNT MCP tests"
else
  warn "only $MCP_COUNT MCP tests (expected ≥5)"
fi

# ── Gate 4: Code quality ────────────────────────────────────────────────
section "4" "Code quality"

if cargo fmt --all -- --check 2>/dev/null; then
  green "formatting clean"
else
  err "formatting issues (run cargo fmt)"
fi

if cargo clippy --workspace --all-targets -- -D warnings 2>/dev/null; then
  green "clippy clean"
else
  err "clippy warnings present"
fi

# ── Gate 5: Documentation ──────────────────────────────────────────────
section "5" "Documentation"

for file in README.md CHANGELOG.md; do
  if [[ -f "$file" ]]; then
    green "$file present"
  else
    err "$file missing"
  fi
done

# Accept LICENSE or LICENSE-MIT
if [[ -f LICENSE-MIT ]]; then
  green "LICENSE-MIT present"
elif [[ -f LICENSE ]]; then
  green "LICENSE present (MIT)"
else
  err "LICENSE missing (expected LICENSE or LICENSE-MIT)"
fi

if cargo doc --workspace --no-deps 2>/dev/null; then
  green "cargo doc builds"
else
  err "cargo doc failed"
fi

if [[ -f CHANGELOG.md ]]; then
  if grep -q "\[${CARGO_VER}\]\|Unreleased" CHANGELOG.md; then
    green "CHANGELOG entry for $CARGO_VER"
  else
    warn "CHANGELOG missing entry for $CARGO_VER"
  fi
fi

# ── Gate 6: Bindings readiness ──────────────────────────────────────────
section "6" "Bindings readiness"

# FFI
if [[ -d crates/agentic-planning-ffi/src ]]; then
  FFI_EXPORTS=$(grep -c '#\[no_mangle\]' crates/agentic-planning-ffi/src/*.rs 2>/dev/null || true)
  if [[ $FFI_EXPORTS -gt 0 ]]; then
    green "FFI: $FFI_EXPORTS exported functions"
  else
    warn "FFI: no #[no_mangle] exports found"
  fi
fi

# Python
if [[ -f python/pyproject.toml ]]; then
  green "Python package: pyproject.toml present"
  if [[ -d python/agentic_planning ]]; then
    green "Python module directory exists"
  else
    warn "Python module directory missing"
  fi
fi

# npm/WASM
if [[ -f npm/wasm/pkg/package.json ]]; then
  green "npm package: package.json present"
  if [[ -f npm/wasm/Cargo.toml ]]; then
    green "WASM crate: Cargo.toml present"
  else
    err "WASM crate: Cargo.toml missing"
  fi
fi

# ── Gate 7: Installer verification ──────────────────────────────────────
section "7" "Installer verification"

if [[ -f scripts/install.sh ]]; then
  green "install.sh exists"

  # Check profile support
  if grep -q "PROFILE" scripts/install.sh; then
    green "profile support present"
  else
    err "no profile support in installer"
  fi

  # Check MCP auto-config
  if grep -q "MCP\|mcpServers" scripts/install.sh; then
    green "MCP auto-configuration present"
  else
    warn "no MCP auto-config in installer"
  fi

  # Check dry-run support
  if grep -q "DRY_RUN\|dry.run" scripts/install.sh; then
    green "dry-run support present"
  else
    warn "no dry-run support"
  fi

  # Dry-run test (strip ANSI codes before grep)
  if bash scripts/install.sh --dry-run --profile desktop 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -qi "complete"; then
    green "installer dry-run succeeds (desktop)"
  else
    err "installer dry-run failed"
  fi
else
  err "scripts/install.sh missing"
fi

# ── Gate 8: Release script ──────────────────────────────────────────────
section "8" "Release automation"

if [[ -f scripts/release.sh ]]; then
  green "release.sh exists"

  # Check it handles version bumps
  if grep -q "VERSION" scripts/release.sh; then
    green "version bump support"
  else
    warn "no version bump logic"
  fi

  # Check it has dry-run default
  if grep -q 'DRY_RUN.*true' scripts/release.sh; then
    green "dry-run enabled by default"
  else
    warn "dry-run not default"
  fi

  # Check crates.io publish
  if grep -q "cargo publish" scripts/release.sh; then
    green "crates.io publish command present"
  else
    warn "no crates.io publish command"
  fi
else
  err "scripts/release.sh missing"
fi

# ── Gate 9: Security hardening ──────────────────────────────────────────
section "9" "Security hardening"

SRC="crates/agentic-planning/src"
MCP_SRC="crates/agentic-planning-mcp/src"

if grep -rq "TokenAuth\|AuthMode" "$SRC" --include="*.rs" 2>/dev/null; then
  green "token authentication present"
else
  err "no token auth"
fi

if grep -rq "FileLock\|StartupLock" "$SRC" --include="*.rs" 2>/dev/null; then
  green "file locking present"
else
  err "no file locking"
fi

if grep -rq "AuditLog\|audit_log" "$SRC" --include="*.rs" 2>/dev/null; then
  green "audit logging present"
else
  err "no audit logging"
fi

if grep -rq "ctrlc\|signal\|shutdown" "$MCP_SRC" --include="*.rs" 2>/dev/null; then
  green "graceful shutdown present"
else
  warn "no graceful shutdown handling"
fi

if grep -rq "sync_all\|rename\|atomic" "$SRC" --include="*.rs" 2>/dev/null; then
  green "atomic writes present"
else
  err "no atomic write support"
fi

# ── Gate 10: Publishing credentials check ───────────────────────────────
section "10" "Publishing credentials (non-blocking)"

# crates.io
if cargo login --help &>/dev/null 2>&1; then
  if [[ -f "${HOME}/.cargo/credentials.toml" ]] || [[ -f "${HOME}/.cargo/credentials" ]]; then
    green "crates.io credentials file exists"
  else
    warn "crates.io: no credentials file (run 'cargo login')"
  fi
fi

# npm
if command -v npm &>/dev/null; then
  if npm whoami &>/dev/null 2>&1; then
    NPM_USER=$(npm whoami 2>/dev/null || echo "unknown")
    green "npm: logged in as $NPM_USER"
  else
    warn "npm: not logged in (run 'npm login')"
  fi
else
  warn "npm: not installed"
fi

# PyPI
if command -v twine &>/dev/null; then
  green "twine installed (PyPI upload tool)"
else
  warn "twine not installed (pip install twine)"
fi

# ── Summary ─────────────────────────────────────────────────────────────
echo ""
bold "╔══════════════════════════════════════════════════════════╗"
bold "║              RELEASE GATE SUMMARY                       ║"
bold "╚══════════════════════════════════════════════════════════╝"
echo ""

if [[ $ERRORS -eq 0 ]]; then
  printf "\033[0;32m  All release gates passed! (%d warnings)\033[0m\n" "$WARNINGS"
  echo ""
  echo "  Ready to publish with:"
  echo "    DRY_RUN=false bash scripts/release.sh ${CARGO_VER}"
else
  printf "\033[0;31m  %d gate failure(s), %d warning(s)\033[0m\n" "$ERRORS" "$WARNINGS"
  echo ""
  echo "  Fix errors above before publishing."
fi

echo ""
echo "  Gate 1:  Version Consistency"
echo "  Gate 2:  Build Verification"
echo "  Gate 3:  Full Test Suite"
echo "  Gate 4:  Code Quality"
echo "  Gate 5:  Documentation"
echo "  Gate 6:  Bindings Readiness"
echo "  Gate 7:  Installer Verification"
echo "  Gate 8:  Release Automation"
echo "  Gate 9:  Security Hardening"
echo "  Gate 10: Publishing Credentials"
echo ""

if [[ $ERRORS -gt 0 ]]; then
  exit 1
fi
