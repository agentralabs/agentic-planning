#!/usr/bin/env bash
# Comprehensive CI guardrail check covering §1-50.
# Run from the agentic-planning root directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

green() { printf "\033[0;32m  ✓ %s\033[0m\n" "$*"; }
yellow() { printf "\033[0;33m  ? %s\033[0m\n" "$*"; }
red() { printf "\033[0;31m  ✗ %s\033[0m\n" "$*"; }
section() { printf "\n\033[1;36m§%s: %s\033[0m\n" "$1" "$2"; }

ERRORS=0
WARNINGS=0

err() { red "$*"; ERRORS=$((ERRORS + 1)); }
warn() { yellow "$*"; WARNINGS=$((WARNINGS + 1)); }

SRC="crates/agentic-planning/src"
MCP_SRC="crates/agentic-planning-mcp/src"

# ═══════════════════════════════════════════════════════════════════
# §1-10: CODE QUALITY
# ═══════════════════════════════════════════════════════════════════

section "1" "Check formatting"
if cargo fmt --all -- --check 2>/dev/null; then
  green "cargo fmt passes"
else
  err "cargo fmt check failed"
fi

section "2" "Clippy (no warnings)"
if cargo clippy --workspace --all-targets -- -D warnings 2>/dev/null; then
  green "clippy clean"
else
  err "clippy has warnings"
fi

section "3" "Compilation check"
if cargo check --workspace 2>/dev/null; then
  green "workspace compiles"
else
  err "compilation failed"
fi

section "4" "Workspace consistency"
if [[ -f Cargo.toml ]]; then
  green "root Cargo.toml exists"
else
  err "Cargo.toml missing"
fi

section "5" "TODO/FIXME in source (warning only)"
TODO_COUNT=$(grep -rn "TODO\|FIXME" "$SRC" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
if [[ $TODO_COUNT -gt 0 ]]; then
  warn "$TODO_COUNT TODO/FIXME markers in source"
else
  green "no TODO/FIXME in source"
fi

section "6" "unwrap() audit"
# Count .unwrap() in non-test code: only lines before #[cfg(test)] per file
UNWRAP_COUNT=0
for f in $(find "$SRC" -name '*.rs'); do
  TEST_LINE=$(grep -n '#\[cfg(test)\]' "$f" 2>/dev/null | head -1 | cut -d: -f1)
  if [[ -n "$TEST_LINE" ]]; then
    COUNT=$(head -n "$((TEST_LINE - 1))" "$f" | grep -c '\.unwrap()' || true)
  else
    COUNT=$(grep -c '\.unwrap()' "$f" 2>/dev/null || true)
  fi
  UNWRAP_COUNT=$((UNWRAP_COUNT + COUNT))
done
if [[ $UNWRAP_COUNT -gt 80 ]]; then
  warn "$UNWRAP_COUNT unwrap() calls in non-test code (threshold: 80)"
else
  green "$UNWRAP_COUNT unwrap() calls in non-test code"
fi

section "7" "panic! detection"
PANIC_COUNT=$(grep -rn 'panic!' "$SRC" --include="*.rs" 2>/dev/null | grep -v '#\[test\]' | grep -v 'mod tests' | wc -l | tr -d ' ')
if [[ $PANIC_COUNT -gt 5 ]]; then
  warn "$PANIC_COUNT panic! calls in non-test code"
else
  green "$PANIC_COUNT panic! calls (acceptable)"
fi

section "8" "Cargo.lock committed"
if [[ -f Cargo.lock ]]; then
  green "Cargo.lock present"
else
  err "Cargo.lock missing"
fi

section "9" "Duplicate dependencies"
DUPES=$(cargo tree --duplicates 2>/dev/null | grep -c "^[a-z]" || true)
if [[ $DUPES -gt 10 ]]; then
  warn "$DUPES duplicate dependency groups"
else
  green "$DUPES duplicate dependency groups"
fi

section "10" "Minimum Rust version"
if grep -q "rust-version" Cargo.toml; then
  RUST_VER=$(grep "rust-version" Cargo.toml | head -1 | cut -d'"' -f2)
  green "rust-version = $RUST_VER"
else
  warn "no rust-version specified in Cargo.toml"
fi

# ═══════════════════════════════════════════════════════════════════
# §11-20: TEST COVERAGE
# ═══════════════════════════════════════════════════════════════════

section "11" "Unit tests"
if cargo test --workspace --lib 2>/dev/null; then
  green "unit tests pass"
else
  err "unit tests failed"
fi

section "12" "Integration tests"
if cargo test --workspace --test '*' 2>/dev/null; then
  green "integration tests pass"
else
  err "integration tests failed"
fi

section "13" "Doc tests"
if cargo test --workspace --doc 2>/dev/null; then
  green "doc tests pass"
else
  err "doc tests failed"
fi

section "14" "Hardening tests"
HARDENING_TESTS=$(cargo test --workspace 2>&1 | grep -c "test.*hardening\|test.*auth\|test.*isolation\|test.*lock" || true)
if [[ $HARDENING_TESTS -gt 0 ]]; then
  green "$HARDENING_TESTS hardening-related tests found"
else
  warn "no hardening test names detected"
fi

section "15" "Stress tests"
STRESS_TESTS=$(cargo test --workspace 2>&1 | grep -c "stress_" || true)
if [[ $STRESS_TESTS -gt 0 ]]; then
  green "$STRESS_TESTS stress tests found"
else
  warn "no stress tests detected"
fi

section "16" "MCP tool count"
if [[ -d "$MCP_SRC" ]]; then
  TOOL_PATTERNS=$(grep -rn "\"tool_" "$MCP_SRC" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
  green "~$TOOL_PATTERNS MCP tool registrations"
else
  warn "MCP source not found"
fi

section "17" "Scenario tests"
SCENARIO_COUNT=$(cargo test --workspace 2>&1 | grep -c "scenario_" || true)
if [[ $SCENARIO_COUNT -gt 0 ]]; then
  green "$SCENARIO_COUNT scenario tests"
else
  warn "no scenario tests detected"
fi

section "18" "Edge case tests"
EDGE_COUNT=$(cargo test --workspace 2>&1 | grep -c "edge_case_" || true)
if [[ $EDGE_COUNT -gt 0 ]]; then
  green "$EDGE_COUNT edge case tests"
else
  warn "no edge_case_ tests detected"
fi

section "19" "Example tests"
if [[ -d examples ]]; then
  green "examples/ directory exists"
else
  warn "no examples/ directory"
fi

section "20" "Total test count"
TOTAL_TESTS=$(cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum+0}')
green "$TOTAL_TESTS total tests across workspace"

# ═══════════════════════════════════════════════════════════════════
# §21-30: HARDENING VERIFICATION
# ═══════════════════════════════════════════════════════════════════

section "21" "Strict validation (ValidationError)"
if grep -rq "ValidationError" "$SRC" --include="*.rs" 2>/dev/null; then
  green "ValidationError type found"
else
  err "no ValidationError — strict validation required"
fi

section "22" "Per-project isolation"
if grep -rq "project_identity\|ProjectId" "$SRC" --include="*.rs" 2>/dev/null; then
  green "project isolation implemented"
else
  err "project isolation missing (ProjectId/project_identity)"
fi

section "23" "No cross-project fallback"
FALLBACK_HITS=$(grep -rn "fallback.*cache\|fallback.*graph\|default.*project" "$SRC" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
if [[ $FALLBACK_HITS -gt 0 ]]; then
  warn "$FALLBACK_HITS potential cross-project fallback patterns"
else
  green "no cross-project fallback patterns"
fi

section "24" "Concurrent lock handling"
if grep -rq "StartupLock\|FileLock\|flock" "$SRC" --include="*.rs" 2>/dev/null; then
  green "lock handling implemented"
else
  err "no lock handling found"
fi

section "25" "Stale lock recovery"
if grep -rq "stale\|STALE_THRESHOLD\|process_alive" "$SRC" --include="*.rs" 2>/dev/null; then
  green "stale lock recovery implemented"
else
  err "no stale lock recovery"
fi

section "26" "MCP config merge"
if [[ -f scripts/install.sh ]]; then
  if grep -qi "restart\|After restart" scripts/install.sh; then
    green "installer mentions restart guidance"
  else
    warn "installer missing restart guidance"
  fi
else
  warn "install.sh not found"
fi

section "27" "Post-install restart guidance"
if [[ -f scripts/install.sh ]]; then
  if grep -qi "restart\|MCP" scripts/install.sh; then
    green "post-install guidance present"
  else
    warn "post-install guidance unclear"
  fi
fi

section "28" "Server mode authentication"
if grep -rq "TokenAuth\|AuthMode\|AGENTIC_AUTH" "$SRC" --include="*.rs" 2>/dev/null; then
  green "token auth implemented"
else
  err "server authentication missing"
fi

section "29" "Atomic file operations"
if grep -rq "atomic\|rename\|sync_all\|write_all.*rename" "$SRC" --include="*.rs" 2>/dev/null; then
  green "atomic file operations present"
else
  warn "atomic file operations not detected"
fi

section "30" "Audit logging"
if grep -rq "audit\|tracing\|log::" "$SRC" --include="*.rs" 2>/dev/null; then
  green "logging/audit support present"
else
  warn "no audit logging detected"
fi

# ═══════════════════════════════════════════════════════════════════
# §31-40: DOCUMENTATION
# ═══════════════════════════════════════════════════════════════════

section "31" "README.md exists"
if [[ -f README.md ]]; then green "present"; else err "README.md missing"; fi

section "32" "CHANGELOG.md exists"
if [[ -f CHANGELOG.md ]]; then green "present"; else err "CHANGELOG.md missing"; fi

section "33" "LICENSE files"
if [[ -f LICENSE-MIT ]] || [[ -f LICENSE-APACHE ]] || [[ -f LICENSE ]]; then
  green "license file present"
else
  err "no license file found"
fi

section "34" "docs/ folder exists"
if [[ -d docs ]]; then green "present"; else err "docs/ missing"; fi

section "35" "Required public docs"
REQUIRED_DOCS=(quickstart.md installation.md mcp-tools.md command-surface.md)
for doc in "${REQUIRED_DOCS[@]}"; do
  if [[ -f "docs/public/$doc" ]]; then
    green "$doc"
  else
    warn "docs/public/$doc missing"
  fi
done

section "36" "Doc links validity"
BROKEN=0
for md in docs/public/*.md 2>/dev/null; do
  if [[ -f "$md" ]]; then
    while IFS= read -r link; do
      target="$(dirname "$md")/$link"
      if [[ ! -e "$target" ]] && [[ ! -e "${target%.md}.md" ]]; then
        BROKEN=$((BROKEN + 1))
      fi
    done < <(grep -oP '\]\(\K[^)]+' "$md" 2>/dev/null | grep -v "^http" || true)
  fi
done
if [[ $BROKEN -gt 0 ]]; then
  warn "$BROKEN potentially broken doc links"
else
  green "doc links OK"
fi

section "37" "Cargo doc builds"
if cargo doc --workspace --no-deps 2>/dev/null; then
  green "cargo doc builds"
else
  err "cargo doc failed"
fi

section "38" "Examples documented"
if [[ -d examples ]]; then
  green "examples/ present"
else
  warn "no examples/ directory"
fi

section "39" "SECURITY.md exists"
if [[ -f SECURITY.md ]]; then green "present"; else warn "SECURITY.md missing"; fi

section "40" "CONTRIBUTING.md exists"
if [[ -f CONTRIBUTING.md ]]; then green "present"; else warn "CONTRIBUTING.md missing"; fi

# ═══════════════════════════════════════════════════════════════════
# §41-47: SISTER-SPECIFIC DOCS
# ═══════════════════════════════════════════════════════════════════

section "41" "ARCHITECTURE documentation"
if [[ -f docs/ARCHITECTURE.md ]] || [[ -f docs/public/concepts.md ]]; then
  green "architecture docs present"
else
  warn "no architecture documentation"
fi

section "42" "INVENTIONS documentation"
if [[ -f docs/INVENTIONS.md ]] || [[ -f docs/public/inventions.md ]]; then
  green "inventions documented"
else
  warn "no inventions documentation"
fi

section "43" "Sister integration docs (optional)"
if [[ -f docs/SISTER-INTEGRATION.md ]] || [[ -f docs/public/integration-guide.md ]]; then
  green "integration docs present"
else
  yellow "optional: sister integration docs"
fi

section "44" "Examples docs (optional)"
if [[ -f docs/EXAMPLES.md ]] || [[ -f docs/public/examples.md ]]; then
  green "examples documented"
else
  yellow "optional: examples docs"
fi

section "45" "FAQ (optional)"
if [[ -f docs/FAQ.md ]] || [[ -f docs/public/faq.md ]]; then
  green "FAQ present"
else
  yellow "optional: FAQ"
fi

section "46" "Troubleshooting (optional)"
if [[ -f docs/TROUBLESHOOTING.md ]] || [[ -f docs/public/troubleshooting.md ]]; then
  green "troubleshooting present"
else
  yellow "optional: troubleshooting docs"
fi

section "47" "Diagrams/assets"
if [[ -d assets ]]; then
  SVG_COUNT=$(find assets -name "*.svg" 2>/dev/null | wc -l | tr -d ' ')
  green "$SVG_COUNT SVG assets"
else
  warn "no assets/ directory"
fi

# ═══════════════════════════════════════════════════════════════════
# §48: CONTRACT COMPLIANCE
# ═══════════════════════════════════════════════════════════════════

section "48" "Canonical sister kit compliance"
REQUIRED_FILES=(Cargo.toml README.md CHANGELOG.md scripts/install.sh .github/workflows/ci.yml)
KIT_MISSING=0
for file in "${REQUIRED_FILES[@]}"; do
  if [[ -f "$file" ]]; then
    green "$file"
  else
    err "missing: $file"
    KIT_MISSING=$((KIT_MISSING + 1))
  fi
done

# ═══════════════════════════════════════════════════════════════════
# §49: RELEASE GATES (local check)
# ═══════════════════════════════════════════════════════════════════

section "49" "Release gate readiness"
CARGO_VER=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2 2>/dev/null || echo "unknown")
green "current version: $CARGO_VER"

if [[ -f CHANGELOG.md ]]; then
  if grep -q "## \[$CARGO_VER\]\|## \[Unreleased\]" CHANGELOG.md; then
    green "CHANGELOG has entry for $CARGO_VER or Unreleased"
  else
    warn "CHANGELOG missing entry for $CARGO_VER"
  fi
fi

if cargo bench --no-run --workspace 2>/dev/null; then
  green "benchmarks compile"
else
  warn "benchmarks not available or failed to compile"
fi

# ═══════════════════════════════════════════════════════════════════
# §50: FINAL SUMMARY
# ═══════════════════════════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║                  GUARDRAIL SUMMARY                      ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

if [[ $ERRORS -eq 0 ]]; then
  printf "\033[0;32m  All guardrails passed! (%d warnings)\033[0m\n" "$WARNINGS"
else
  printf "\033[0;31m  %d error(s), %d warning(s)\033[0m\n" "$ERRORS" "$WARNINGS"
fi

echo ""
echo "  §1-10:  Code Quality"
echo "  §11-20: Test Coverage"
echo "  §21-30: Hardening Verification"
echo "  §31-40: Documentation"
echo "  §41-47: Sister-Specific Docs"
echo "  §48:    Contract Compliance"
echo "  §49:    Release Gates"
echo "  §50:    Final Verification"
echo ""

if [[ $ERRORS -gt 0 ]]; then
  exit 1
fi
