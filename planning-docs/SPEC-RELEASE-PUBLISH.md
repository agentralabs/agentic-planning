# SPEC-RELEASE-PUBLISH

> **Scope:** ALL Sisters
> **Status:** Canonical Standard
> **Enforced:** CI Guardrails

---

## 1. Overview

Every sister MUST follow the release and publish workflow to ensure:
- Consistent versioning across all packages
- All registries updated atomically
- Documentation stays in sync
- Release notes are comprehensive
- Rollback is possible

---

## 2. Version Strategy

```
VERSIONING:
═══════════

Semantic Versioning: MAJOR.MINOR.PATCH

0.x.x  - Development (breaking changes allowed)
1.x.x  - Stable (breaking changes = major bump)

VERSION SOURCES (single source of truth):
  - Cargo.toml (workspace.package.version)
  - Propagates to: package.json, pyproject.toml, CHANGELOG.md

PRE-RELEASE TAGS:
  - 0.1.0-alpha.1
  - 0.1.0-beta.1
  - 0.1.0-rc.1
```

---

## 3. Release Checklist

```
PRE-RELEASE CHECKLIST:
══════════════════════

□ All CI checks pass (cargo test, clippy, fmt)
□ All hardening tests pass
□ MCP tool count matches expectation
□ CHANGELOG.md updated with release notes
□ Version bumped in Cargo.toml
□ Documentation updated
□ Examples tested
□ README updated if needed
□ Breaking changes documented
```

---

## 4. Release Script

```bash
#!/usr/bin/env bash
# scripts/release.sh
# Usage: ./scripts/release.sh 0.2.0

set -euo pipefail

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "Invalid version format: $VERSION"
    echo "Expected: MAJOR.MINOR.PATCH[-prerelease]"
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              Release Script for v${VERSION}                       "
echo "╚════════════════════════════════════════════════════════════════╝"

# ============================================================================
# PRE-FLIGHT CHECKS
# ============================================================================

echo ""
echo "=== Pre-flight checks ==="

# Check we're on main branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
    echo "ERROR: Must be on main/master branch (currently on $BRANCH)"
    exit 1
fi

# Check working directory is clean
if ! git diff --quiet || ! git diff --staged --quiet; then
    echo "ERROR: Working directory has uncommitted changes"
    git status --short
    exit 1
fi

# Check upstream is up to date
git fetch origin
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/$BRANCH)
if [[ "$LOCAL" != "$REMOTE" ]]; then
    echo "ERROR: Local branch differs from remote"
    exit 1
fi

# Check CI status
echo "Checking CI status..."
if command -v gh &> /dev/null; then
    if ! gh run list --limit 1 --json conclusion | grep -q '"conclusion":"success"'; then
        echo "WARNING: Latest CI run may not have succeeded"
        read -rp "Continue anyway? [y/N] " response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

echo "Pre-flight checks passed!"

# ============================================================================
# VERSION BUMP
# ============================================================================

echo ""
echo "=== Bumping version to $VERSION ==="

# Update Cargo.toml (workspace)
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
cargo update -w

# Update Python bindings if exist
if [[ -f "bindings/python/pyproject.toml" ]]; then
    sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" bindings/python/pyproject.toml
    rm bindings/python/pyproject.toml.bak
fi

# Update Node bindings if exist
if [[ -f "bindings/node/package.json" ]]; then
    npm --prefix bindings/node version "$VERSION" --no-git-tag-version
fi

# ============================================================================
# CHANGELOG
# ============================================================================

echo ""
echo "=== Updating CHANGELOG ==="

# Check if changelog entry exists
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo "ERROR: CHANGELOG.md missing entry for [$VERSION]"
    echo ""
    echo "Please add a changelog entry like:"
    echo ""
    echo "## [$VERSION] - $(date +%Y-%m-%d)"
    echo ""
    echo "### Added"
    echo "- ..."
    echo ""
    exit 1
fi

# ============================================================================
# BUILD AND TEST
# ============================================================================

echo ""
echo "=== Building and testing ==="

cargo build --release
cargo test --release
cargo clippy -- -D warnings

# ============================================================================
# COMMIT AND TAG
# ============================================================================

echo ""
echo "=== Committing version bump ==="

git add Cargo.toml Cargo.lock CHANGELOG.md
[[ -f "bindings/python/pyproject.toml" ]] && git add bindings/python/pyproject.toml
[[ -f "bindings/node/package.json" ]] && git add bindings/node/package.json

git commit -m "release: v${VERSION}"

echo ""
echo "=== Creating tag ==="

# Create annotated tag with changelog excerpt
CHANGELOG_EXCERPT=$(awk "/^## \[$VERSION\]/,/^## \[/" CHANGELOG.md | head -n -1)
git tag -a "v${VERSION}" -m "Release v${VERSION}

${CHANGELOG_EXCERPT}"

# ============================================================================
# PUSH
# ============================================================================

echo ""
echo "=== Pushing to origin ==="

git push origin $BRANCH
git push origin "v${VERSION}"

# ============================================================================
# PUBLISH
# ============================================================================

echo ""
echo "=== Publishing packages ==="

# Publish to crates.io
echo "Publishing to crates.io..."
cargo publish -p agentic-${SISTER_NAME} --allow-dirty
cargo publish -p agentic-${SISTER_NAME}-mcp --allow-dirty

# Publish to PyPI if bindings exist
if [[ -f "bindings/python/pyproject.toml" ]]; then
    echo "Publishing to PyPI..."
    cd bindings/python
    maturin publish
    cd ../..
fi

# Publish to npm if bindings exist
if [[ -f "bindings/node/package.json" ]]; then
    echo "Publishing to npm..."
    cd bindings/node
    npm publish
    cd ../..
fi

# ============================================================================
# GITHUB RELEASE
# ============================================================================

echo ""
echo "=== Creating GitHub release ==="

if command -v gh &> /dev/null; then
    gh release create "v${VERSION}" \
        --title "v${VERSION}" \
        --notes "${CHANGELOG_EXCERPT}" \
        --latest
    
    # Upload binaries
    echo "Uploading release assets..."
    for platform in darwin-aarch64 darwin-x86_64 linux-x86_64; do
        if [[ -f "target/release/${BINARY_NAME}-${platform}.tar.gz" ]]; then
            gh release upload "v${VERSION}" "target/release/${BINARY_NAME}-${platform}.tar.gz"
        fi
    done
else
    echo "GitHub CLI not installed. Create release manually at:"
    echo "https://github.com/${REPO}/releases/new?tag=v${VERSION}"
fi

# ============================================================================
# COMPLETE
# ============================================================================

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              Release v${VERSION} Complete!                        "
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Published to:"
echo "  • crates.io: https://crates.io/crates/agentic-${SISTER_NAME}"
[[ -f "bindings/python/pyproject.toml" ]] && echo "  • PyPI: https://pypi.org/project/agentic-${SISTER_NAME}"
[[ -f "bindings/node/package.json" ]] && echo "  • npm: https://npmjs.com/package/agentic-${SISTER_NAME}"
echo "  • GitHub: https://github.com/${REPO}/releases/tag/v${VERSION}"
echo ""
```

---

## 5. Release Gate Checklist

Every release MUST pass these gates before publish:

```rust
//! tests/release_gates.rs

/// Gate 1: Version consistency
#[test]
fn gate_version_consistency() {
    let cargo_version = env!("CARGO_PKG_VERSION");
    
    // Check Python version if exists
    if let Ok(py_version) = read_pyproject_version() {
        assert_eq!(cargo_version, py_version, "Python version mismatch");
    }
    
    // Check Node version if exists
    if let Ok(node_version) = read_package_json_version() {
        assert_eq!(cargo_version, node_version, "Node version mismatch");
    }
}

/// Gate 2: Changelog entry exists
#[test]
fn gate_changelog_entry() {
    let version = env!("CARGO_PKG_VERSION");
    let changelog = include_str!("../../CHANGELOG.md");
    
    assert!(
        changelog.contains(&format!("## [{}]", version)),
        "CHANGELOG.md missing entry for version {}", version
    );
}

/// Gate 3: All hardening tests pass
#[test]
fn gate_hardening_tests() {
    // This is a meta-test that ensures hardening tests exist
    let test_files = [
        "tests/hardening_tests.rs",
        "tests/stress_hardening.rs",
    ];
    
    for file in test_files {
        assert!(
            std::path::Path::new(file).exists(),
            "Missing required test file: {}", file
        );
    }
}

/// Gate 4: MCP tool count matches
#[test]
fn gate_mcp_tool_count() {
    let expected_tools = 12;  // Update per sister
    let server = McpServer::new();
    let tools = server.tools();
    
    assert_eq!(
        tools.len(), expected_tools,
        "MCP tool count mismatch: expected {}, got {}", 
        expected_tools, tools.len()
    );
}

/// Gate 5: No TODO/FIXME in release
#[test]
fn gate_no_todo_fixme() {
    let src_dir = std::path::Path::new("src");
    let mut violations = vec![];
    
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension() == Some("rs".as_ref()) {
            let contents = std::fs::read_to_string(entry.path()).unwrap();
            for (line_num, line) in contents.lines().enumerate() {
                if line.contains("TODO") || line.contains("FIXME") {
                    violations.push(format!(
                        "{}:{}: {}", 
                        entry.path().display(), 
                        line_num + 1, 
                        line.trim()
                    ));
                }
            }
        }
    }
    
    assert!(
        violations.is_empty(),
        "Found TODO/FIXME in release code:\n{}", 
        violations.join("\n")
    );
}

/// Gate 6: Documentation build succeeds
#[test]
fn gate_docs_build() {
    let output = std::process::Command::new("cargo")
        .args(["doc", "--no-deps"])
        .output()
        .expect("Failed to run cargo doc");
    
    assert!(
        output.status.success(),
        "Documentation build failed:\n{}", 
        String::from_utf8_lossy(&output.stderr)
    );
}
```

---

## 6. Multi-Registry Publishing

```
REGISTRY MATRIX:
════════════════

┌────────────┬─────────────────────────────────────────────────────────────────┐
│ Registry   │ Package                                                         │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ crates.io  │ agentic-{sister}                                               │
│            │ agentic-{sister}-mcp                                           │
│            │ agentic-{sister}-bridges                                       │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ PyPI       │ agentic-{sister} (maturin build)                               │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ npm        │ @agentic/{sister}                                              │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ GitHub     │ Binary releases (darwin-arm64, darwin-x64, linux-x64)          │
└────────────┴─────────────────────────────────────────────────────────────────┘
```

---

## 7. Rollback Procedure

```bash
#!/usr/bin/env bash
# scripts/rollback.sh
# Usage: ./scripts/rollback.sh 0.1.5

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version-to-rollback-to>"
    exit 1
fi

echo "Rolling back to v${VERSION}..."

# Yank crates.io versions
cargo yank --version "$CURRENT_VERSION" -p agentic-${SISTER_NAME}
cargo yank --version "$CURRENT_VERSION" -p agentic-${SISTER_NAME}-mcp

# Mark GitHub release as not latest
gh release edit "v${CURRENT_VERSION}" --latest=false
gh release edit "v${VERSION}" --latest

echo "Rollback complete. Users will now get v${VERSION} by default."
echo ""
echo "IMPORTANT: The yanked version is still installable if explicitly requested."
echo "For complete removal, contact the respective registry maintainers."
```

---

## 8. Automated Release (GitHub Actions)

```yaml
# .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Run release gates
        run: cargo test --test release_gates
  
  build:
    needs: release-gates
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar -czvf agentic-${SISTER_NAME}-${{ matrix.target }}.tar.gz agentic-${SISTER_NAME}
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/*.tar.gz
  
  publish-crates:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo publish -p agentic-${SISTER_NAME}
          sleep 30  # Wait for crates.io to index
          cargo publish -p agentic-${SISTER_NAME}-mcp
  
  publish-pypi:
    needs: build
    runs-on: ubuntu-latest
    if: hashFiles('bindings/python/pyproject.toml') != ''
    steps:
      - uses: actions/checkout@v4
      
      - name: Install maturin
        run: pip install maturin
      
      - name: Publish to PyPI
        env:
          MATURIN_PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
        run: |
          cd bindings/python
          maturin publish
  
  publish-npm:
    needs: build
    runs-on: ubuntu-latest
    if: hashFiles('bindings/node/package.json') != ''
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      
      - name: Publish to npm
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          cd bindings/node
          npm publish
  
  github-release:
    needs: [build, publish-crates]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Download artifacts
        uses: actions/download-artifact@v4
      
      - name: Extract changelog
        id: changelog
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          NOTES=$(awk "/^## \[$VERSION\]/,/^## \[/" CHANGELOG.md | head -n -1)
          echo "notes<<EOF" >> $GITHUB_OUTPUT
          echo "$NOTES" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT
      
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          body: ${{ steps.changelog.outputs.notes }}
          files: |
            binary-*/agentic-*.tar.gz
```

---

## 9. Post-Release

```
POST-RELEASE TASKS:
═══════════════════

□ Verify all registries updated
  - crates.io: https://crates.io/crates/agentic-{sister}
  - PyPI: https://pypi.org/project/agentic-{sister}
  - npm: https://npmjs.com/package/@agentic/{sister}
  
□ Verify installer pulls new version
  - curl -fsSL https://agentic.sh/{sister} | bash
  
□ Update documentation site
  - Run docs-sync workflow
  
□ Announce release
  - Discord
  - Twitter/X
  - GitHub Discussions
  
□ Monitor for issues
  - Watch GitHub issues for 24-48 hours
  - Check crates.io download stats
```

---

*Document: SPEC-RELEASE-PUBLISH.md*
*Applies to: ALL Sisters*
