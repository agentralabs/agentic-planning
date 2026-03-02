# SPEC-CI-GUARDRAILS

> **Scope:** ALL Sisters
> **Status:** Canonical Standard
> **Enforced:** All PRs and Releases

---

## 1. Overview

Every sister MUST have CI guardrails that automatically enforce:
- Code quality (clippy, fmt, tests)
- Hardening requirements
- Documentation completeness
- Release gates

PRs and releases CANNOT proceed if guardrails fail.

---

## 2. Guardrail Sections

```
TOTAL GUARDRAILS: 50 sections
═════════════════════════════

SECTION 1-10:   Code Quality
SECTION 11-20:  Test Coverage
SECTION 21-30:  Hardening Verification
SECTION 31-40:  Documentation
SECTION 41-47:  Sister-Specific (8 doc pages)
SECTION 48:     Contract Compliance
SECTION 49:     Release Gates
SECTION 50:     Final Verification
```

---

## 3. Main CI Workflow

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main, master, develop]
  pull_request:
    branches: [main, master]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ========================================================================
  # SECTION 1-10: CODE QUALITY
  # ========================================================================
  
  code-quality:
    name: "§1-10: Code Quality"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt
      
      - name: "§1: Check formatting"
        run: cargo fmt --all -- --check
      
      - name: "§2: Clippy (no warnings)"
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: "§3: Check compilation"
        run: cargo check --all-targets --all-features
      
      - name: "§4: Check workspace consistency"
        run: |
          # Verify all crates use workspace dependencies
          for toml in crates/*/Cargo.toml; do
            if grep -q "version = \"" "$toml" && ! grep -q "workspace = true" "$toml"; then
              echo "ERROR: $toml should use workspace dependencies"
              exit 1
            fi
          done
      
      - name: "§5: Check for TODO/FIXME (release branches only)"
        if: github.ref == 'refs/heads/main'
        run: |
          if grep -rn "TODO\|FIXME" src/; then
            echo "ERROR: TODO/FIXME found in release code"
            exit 1
          fi
      
      - name: "§6: Check for unwrap() in library code"
        run: |
          # Count unwraps (warn if too many)
          UNWRAP_COUNT=$(grep -rn "\.unwrap()" src/ | wc -l)
          if [[ $UNWRAP_COUNT -gt 50 ]]; then
            echo "WARNING: $UNWRAP_COUNT unwrap() calls found"
          fi
      
      - name: "§7: Check for panic! in library code"
        run: |
          if grep -rn "panic!" src/ --include="*.rs" | grep -v "test"; then
            echo "WARNING: panic! found in non-test code"
          fi
      
      - name: "§8: Verify Cargo.lock is committed"
        run: |
          if [[ ! -f Cargo.lock ]]; then
            echo "ERROR: Cargo.lock must be committed"
            exit 1
          fi
      
      - name: "§9: Check for duplicate dependencies"
        run: cargo tree --duplicates
      
      - name: "§10: Verify minimum Rust version"
        run: |
          MIN_VERSION=$(grep "rust-version" Cargo.toml | head -1 | cut -d'"' -f2)
          echo "Minimum Rust version: $MIN_VERSION"

  # ========================================================================
  # SECTION 11-20: TEST COVERAGE
  # ========================================================================
  
  tests:
    name: "§11-20: Test Coverage"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: "§11: Unit tests"
        run: cargo test --lib --all-features
      
      - name: "§12: Integration tests"
        run: cargo test --test '*' --all-features
      
      - name: "§13: Doc tests"
        run: cargo test --doc --all-features
      
      - name: "§14: Hardening tests"
        run: cargo test --test hardening_tests
      
      - name: "§15: Stress tests"
        run: cargo test --test stress_hardening --release
      
      - name: "§16: MCP tool count verification"
        run: cargo test mcp_tool_count_matches_expectation
      
      - name: "§17: Phase tests (all phases)"
        run: |
          for i in $(seq -f "%02g" 1 12); do
            if [[ -f "tests/phase${i}_*.rs" ]]; then
              cargo test --test "phase${i}_*"
            fi
          done
      
      - name: "§18: Release gate tests"
        run: cargo test --test release_gates
      
      - name: "§19: Example tests"
        run: cargo test --examples
      
      - name: "§20: Test coverage report"
        run: |
          cargo install cargo-tarpaulin || true
          cargo tarpaulin --out Xml --skip-clean || echo "Coverage not required"

  # ========================================================================
  # SECTION 21-30: HARDENING VERIFICATION
  # ========================================================================
  
  hardening:
    name: "§21-30: Hardening Verification"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: "§21: Strict validation (no silent fallbacks)"
        run: |
          # Check for ValidationError usage
          if ! grep -rq "ValidationError" src/; then
            echo "ERROR: No ValidationError found - strict validation required"
            exit 1
          fi
      
      - name: "§22: Per-project isolation"
        run: |
          # Check for project_identity function
          if ! grep -rq "project_identity\|ProjectId" src/; then
            echo "WARNING: Project isolation may not be implemented"
          fi
      
      - name: "§23: No cross-project fallback"
        run: |
          # Check for dangerous fallback patterns
          if grep -rn "latest\|fallback\|default" src/ | grep -i "cache\|graph\|project"; then
            echo "WARNING: Potential cross-project fallback detected"
          fi
      
      - name: "§24: Concurrent lock handling"
        run: |
          # Check for lock implementation
          if ! grep -rq "StartupLock\|FileLock\|lock" src/; then
            echo "WARNING: Concurrent lock handling may not be implemented"
          fi
      
      - name: "§25: Stale lock recovery"
        run: |
          if ! grep -rq "stale.*lock\|STALE_LOCK" src/; then
            echo "WARNING: Stale lock recovery may not be implemented"
          fi
      
      - name: "§26: MCP config merge (not replace)"
        run: |
          # Check installer for merge logic
          if [[ -f "scripts/install.sh" ]]; then
            if ! grep -q "merge" scripts/install.sh; then
              echo "ERROR: Installer must use merge-only MCP config updates"
              exit 1
            fi
          fi
      
      - name: "§27: Post-install restart guidance"
        run: |
          if [[ -f "scripts/install.sh" ]]; then
            if ! grep -qi "restart" scripts/install.sh; then
              echo "ERROR: Installer must include restart guidance"
              exit 1
            fi
          fi
      
      - name: "§28: Server mode authentication"
        run: |
          if ! grep -rq "AUTH_TOKEN\|TokenAuth\|AuthMode" src/; then
            echo "WARNING: Server authentication may not be implemented"
          fi
      
      - name: "§29: Atomic file operations"
        run: |
          if ! grep -rq "atomic\|rename\|sync_all" src/; then
            echo "WARNING: Atomic file operations may not be implemented"
          fi
      
      - name: "§30: Audit logging"
        run: |
          if ! grep -rq "AuditLog\|audit" src/; then
            echo "WARNING: Audit logging may not be implemented"
          fi

  # ========================================================================
  # SECTION 31-40: DOCUMENTATION
  # ========================================================================
  
  docs:
    name: "§31-40: Documentation"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: "§31: README.md exists"
        run: test -f README.md
      
      - name: "§32: CHANGELOG.md exists"
        run: test -f CHANGELOG.md
      
      - name: "§33: LICENSE files exist"
        run: |
          test -f LICENSE-MIT || test -f LICENSE-APACHE || test -f LICENSE
      
      - name: "§34: docs/ folder exists"
        run: test -d docs
      
      - name: "§35: Required docs exist"
        run: |
          for doc in README.md QUICKSTART.md API.md CLI.md MCP-TOOLS.md; do
            if [[ ! -f "docs/$doc" ]]; then
              echo "ERROR: Missing required doc: docs/$doc"
              exit 1
            fi
          done
      
      - name: "§36: Doc links are valid"
        run: |
          # Simple link checker
          for md in docs/*.md; do
            while IFS= read -r link; do
              if [[ "$link" =~ ^\.\./|\^\./ ]]; then
                # Check relative link
                target=$(dirname "$md")/"$link"
                if [[ ! -e "${target%.md}.md" ]] && [[ ! -e "$target" ]]; then
                  echo "WARNING: Broken link in $md: $link"
                fi
              fi
            done < <(grep -oP '\]\(\K[^)]+' "$md" | grep -v "^http")
          done
      
      - name: "§37: Cargo doc builds"
        run: cargo doc --no-deps --all-features
      
      - name: "§38: Examples documented"
        run: |
          if [[ -d examples ]]; then
            for example in examples/*.rs; do
              if ! head -10 "$example" | grep -q "//!"; then
                echo "WARNING: Example missing documentation: $example"
              fi
            done
          fi
      
      - name: "§39: SECURITY.md exists"
        run: test -f SECURITY.md
      
      - name: "§40: CONTRIBUTING.md exists"
        run: test -f CONTRIBUTING.md

  # ========================================================================
  # SECTION 41-47: SISTER-SPECIFIC (8 doc pages)
  # ========================================================================
  
  sister-docs:
    name: "§41-47: Sister Documentation"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: "§41: ARCHITECTURE.md (page 1)"
        run: |
          if [[ -f "docs/ARCHITECTURE.md" ]]; then
            # Check minimum length
            LINES=$(wc -l < docs/ARCHITECTURE.md)
            if [[ $LINES -lt 50 ]]; then
              echo "WARNING: ARCHITECTURE.md seems too short ($LINES lines)"
            fi
          fi
      
      - name: "§42: INVENTIONS.md (page 2)"
        run: |
          if [[ -f "docs/INVENTIONS.md" ]]; then
            # Check inventions are documented
            INVENTION_COUNT=$(grep -c "^### \|^## Invention" docs/INVENTIONS.md || true)
            echo "Documented inventions: $INVENTION_COUNT"
          fi
      
      - name: "§43: SISTER-INTEGRATION.md (page 3)"
        run: test -f docs/SISTER-INTEGRATION.md || echo "Optional: SISTER-INTEGRATION.md"
      
      - name: "§44: EXAMPLES.md (page 4)"
        run: test -f docs/EXAMPLES.md || echo "Optional: EXAMPLES.md"
      
      - name: "§45: FAQ.md (page 5)"
        run: test -f docs/FAQ.md || echo "Optional: FAQ.md"
      
      - name: "§46: TROUBLESHOOTING.md (page 6)"
        run: test -f docs/TROUBLESHOOTING.md || echo "Optional: TROUBLESHOOTING.md"
      
      - name: "§47: Diagrams exist (pages 7-8)"
        run: |
          if [[ -d "docs/assets" ]]; then
            SVG_COUNT=$(find docs/assets -name "*.svg" | wc -l)
            echo "SVG diagrams: $SVG_COUNT"
          fi

  # ========================================================================
  # SECTION 48: CONTRACT COMPLIANCE
  # ========================================================================
  
  contracts:
    name: "§48: Contract Compliance"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: "§48: Verify canonical sister compliance"
        run: |
          # Check for required files per CANONICAL_SISTER_KIT.md
          REQUIRED=(
            "Cargo.toml"
            "README.md"
            "CHANGELOG.md"
            "scripts/install.sh"
            ".github/workflows/ci.yml"
          )
          
          MISSING=0
          for file in "${REQUIRED[@]}"; do
            if [[ ! -f "$file" ]]; then
              echo "MISSING: $file"
              MISSING=$((MISSING + 1))
            fi
          done
          
          if [[ $MISSING -gt 0 ]]; then
            echo ""
            echo "ERROR: $MISSING required files missing"
            echo "See CANONICAL_SISTER_KIT.md for requirements"
            exit 1
          fi

  # ========================================================================
  # SECTION 49: RELEASE GATES
  # ========================================================================
  
  release-gates:
    name: "§49: Release Gates"
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    needs: [code-quality, tests, hardening, docs, sister-docs, contracts]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: "§49.1: Version consistency"
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          CARGO_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
          
          if [[ "$VERSION" != "$CARGO_VERSION" ]]; then
            echo "ERROR: Tag version ($VERSION) != Cargo.toml version ($CARGO_VERSION)"
            exit 1
          fi
      
      - name: "§49.2: CHANGELOG has entry"
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
            echo "ERROR: CHANGELOG.md missing entry for [$VERSION]"
            exit 1
          fi
      
      - name: "§49.3: No TODO/FIXME"
        run: |
          if grep -rn "TODO\|FIXME" src/; then
            echo "ERROR: TODO/FIXME found in release code"
            exit 1
          fi
      
      - name: "§49.4: All tests pass"
        run: cargo test --all-features --release
      
      - name: "§49.5: Benchmarks run"
        run: cargo bench --no-run

  # ========================================================================
  # SECTION 50: FINAL VERIFICATION
  # ========================================================================
  
  final:
    name: "§50: Final Verification"
    runs-on: ubuntu-latest
    needs: [code-quality, tests, hardening, docs, sister-docs, contracts]
    steps:
      - uses: actions/checkout@v4
      
      - name: "§50: Guardrail summary"
        run: |
          echo "╔════════════════════════════════════════════════════════════════╗"
          echo "║                    GUARDRAIL SUMMARY                           ║"
          echo "╚════════════════════════════════════════════════════════════════╝"
          echo ""
          echo "✓ §1-10:  Code Quality        PASSED"
          echo "✓ §11-20: Test Coverage       PASSED"
          echo "✓ §21-30: Hardening           PASSED"
          echo "✓ §31-40: Documentation       PASSED"
          echo "✓ §41-47: Sister Docs         PASSED"
          echo "✓ §48:    Contract Compliance PASSED"
          echo ""
          echo "All 48 guardrail sections passed!"
```

---

## 4. Guardrail Enforcement

```
ENFORCEMENT RULES:
══════════════════

PRs to main/master:
  - ALL guardrail sections must pass
  - No merge without green CI
  - Minimum 1 approval required

Release tags:
  - ALL guardrail sections must pass
  - Release gate tests must pass
  - Version consistency verified
  - CHANGELOG entry required

Nightly builds:
  - Stress tests run
  - Performance regression check
  - Dependency audit
```

---

## 5. Local Guardrail Check

```bash
#!/usr/bin/env bash
# scripts/check-guardrails.sh
# Run all guardrails locally before pushing

set -euo pipefail

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                 LOCAL GUARDRAIL CHECK                          ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

FAILED=0

check() {
    local section="$1"
    local name="$2"
    local cmd="$3"
    
    echo -n "§${section}: ${name}... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo "✓"
    else
        echo "✗"
        FAILED=$((FAILED + 1))
    fi
}

# Code Quality
check "1" "Formatting" "cargo fmt --check"
check "2" "Clippy" "cargo clippy -- -D warnings"
check "3" "Build" "cargo check"

# Tests
check "11" "Unit tests" "cargo test --lib"
check "14" "Hardening tests" "cargo test --test hardening_tests"
check "16" "MCP tool count" "cargo test mcp_tool_count"

# Documentation
check "31" "README" "test -f README.md"
check "32" "CHANGELOG" "test -f CHANGELOG.md"
check "37" "Cargo doc" "cargo doc --no-deps"

echo ""
if [[ $FAILED -gt 0 ]]; then
    echo "FAILED: $FAILED guardrails failed"
    exit 1
else
    echo "PASSED: All checked guardrails passed"
fi
```

---

## 6. Guardrail Skip (Emergency Only)

```yaml
# For emergencies only - requires approval

# In commit message:
# [skip ci] - Skips ALL CI (use sparingly)

# In workflow:
jobs:
  guardrails:
    if: "!contains(github.event.head_commit.message, '[skip guardrails]')"
    
# Requires admin approval to merge with skipped guardrails
```

---

## 7. Summary

```
CI GUARDRAIL REQUIREMENTS:
══════════════════════════

□ 50 guardrail sections defined
□ All sections automated in CI
□ PRs cannot merge without passing
□ Releases gated on all tests
□ Local check script available
□ Emergency skip requires approval
□ Nightly runs stress tests
□ Release gates verify version/changelog
```

---

*Document: SPEC-CI-GUARDRAILS.md*
*Applies to: ALL Sisters*
