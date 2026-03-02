# SPEC-DOCS-PUBLIC-SYNC

> **Scope:** ALL Sisters
> **Status:** Canonical Standard
> **Enforced:** CI Guardrails

---

## 1. Overview

Every sister MUST maintain synchronized documentation across:
- Repository `docs/` folder (source of truth)
- Public documentation site (docs.agentic.so/{sister})
- README.md (summary only)
- crates.io/PyPI/npm package descriptions

Changes to documentation in the repo MUST automatically sync to the public site.

---

## 2. Documentation Structure

```
docs/
├── README.md                 # Overview (synced to docs site home)
├── QUICKSTART.md             # Getting started guide
├── ARCHITECTURE.md           # Technical architecture
├── API.md                    # Core API reference
├── CLI.md                    # CLI command reference
├── MCP-TOOLS.md              # MCP tool documentation
├── INVENTIONS.md             # Sister-specific inventions
├── SISTER-INTEGRATION.md     # Integration with other sisters
├── EXAMPLES.md               # Code examples
├── FAQ.md                    # Frequently asked questions
├── TROUBLESHOOTING.md        # Common issues and solutions
├── CHANGELOG.md              # Symlink to ../CHANGELOG.md
└── assets/
    ├── architecture.svg      # Architecture diagram
    ├── workflow.svg          # Workflow diagram
    └── logo.png              # Sister logo
```

---

## 3. Sync Rules

```
SYNC RULES:
═══════════

SOURCE OF TRUTH: Repository docs/ folder

SYNC TARGETS:
  1. docs.agentic.so/{sister}/*   ← From docs/*.md
  2. crates.io description         ← From README.md (first 3 paragraphs)
  3. PyPI description              ← From README.md
  4. npm description               ← From README.md

SYNC TRIGGERS:
  - Push to main/master branch
  - Release tag creation
  - Manual workflow dispatch

NEVER EDIT DIRECTLY:
  - docs.agentic.so content (generated from repo)
  - Package descriptions (extracted from README)
```

---

## 4. Sync Script

```bash
#!/usr/bin/env bash
# scripts/docs-sync.sh
# Sync documentation to public site

set -euo pipefail

SISTER_NAME="${SISTER_NAME:-planning}"
DOCS_REPO="agentralabs/docs.agentic.so"
DOCS_BRANCH="main"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              Documentation Sync for ${SISTER_NAME}                    "
echo "╚════════════════════════════════════════════════════════════════╝"

# ============================================================================
# SETUP
# ============================================================================

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Clone docs repo
echo "Cloning docs repository..."
git clone --depth 1 "https://github.com/${DOCS_REPO}.git" "$TEMP_DIR/docs-site"

# ============================================================================
# TRANSFORM AND SYNC
# ============================================================================

echo "Syncing documentation..."

TARGET_DIR="$TEMP_DIR/docs-site/content/${SISTER_NAME}"
mkdir -p "$TARGET_DIR"

# Copy and transform markdown files
for md_file in docs/*.md; do
    if [[ -f "$md_file" ]]; then
        filename=$(basename "$md_file")
        
        # Transform for Hugo/docs site
        transform_for_site "$md_file" > "$TARGET_DIR/$filename"
    fi
done

# Copy assets
if [[ -d "docs/assets" ]]; then
    cp -r docs/assets "$TARGET_DIR/"
fi

# Generate navigation
generate_nav "$TARGET_DIR" > "$TARGET_DIR/_index.md"

# ============================================================================
# TRANSFORM FUNCTIONS
# ============================================================================

transform_for_site() {
    local input="$1"
    local filename=$(basename "$input")
    
    # Add Hugo front matter
    cat <<EOF
---
title: "${filename%.md}"
weight: $(get_weight "$filename")
---

EOF
    
    # Transform content
    cat "$input" | \
        # Fix internal links
        sed 's/](\.\/\([^)]*\)\.md)/](\/'"${SISTER_NAME}"'\/\1)/g' | \
        # Fix asset paths
        sed 's/](\.\/assets\//](\/'"${SISTER_NAME}"'\/assets\//g' | \
        # Remove any HTML comments
        sed '/<!--/,/-->/d'
}

get_weight() {
    local filename="$1"
    case "$filename" in
        "README.md")            echo "1" ;;
        "QUICKSTART.md")        echo "2" ;;
        "ARCHITECTURE.md")      echo "3" ;;
        "API.md")               echo "4" ;;
        "CLI.md")               echo "5" ;;
        "MCP-TOOLS.md")         echo "6" ;;
        "INVENTIONS.md")        echo "7" ;;
        "SISTER-INTEGRATION.md") echo "8" ;;
        "EXAMPLES.md")          echo "9" ;;
        "FAQ.md")               echo "10" ;;
        "TROUBLESHOOTING.md")   echo "11" ;;
        "CHANGELOG.md")         echo "99" ;;
        *)                      echo "50" ;;
    esac
}

generate_nav() {
    local dir="$1"
    
    cat <<EOF
---
title: "Agentic ${SISTER_NAME^}"
weight: 1
---

Welcome to the Agentic ${SISTER_NAME^} documentation.

## Contents

EOF
    
    for md_file in "$dir"/*.md; do
        if [[ -f "$md_file" ]] && [[ "$(basename "$md_file")" != "_index.md" ]]; then
            local name=$(basename "$md_file" .md)
            echo "- [${name}](${name}/)"
        fi
    done
}

# ============================================================================
# COMMIT AND PUSH
# ============================================================================

echo "Committing changes..."

cd "$TEMP_DIR/docs-site"

git config user.name "Agentic Bot"
git config user.email "bot@agentic.so"

git add "content/${SISTER_NAME}"

if git diff --staged --quiet; then
    echo "No documentation changes detected."
else
    git commit -m "docs(${SISTER_NAME}): sync from repository

Synced from: $(git -C "$OLDPWD" rev-parse HEAD)
"
    
    echo "Pushing to docs site..."
    git push origin "$DOCS_BRANCH"
fi

echo ""
echo "Documentation sync complete!"
echo "View at: https://docs.agentic.so/${SISTER_NAME}"
```

---

## 5. Documentation Quality Checks

```rust
//! tests/docs_quality.rs

/// Check all documentation links are valid
#[test]
fn check_doc_links() {
    let docs_dir = std::path::Path::new("docs");
    let mut broken_links = vec![];
    
    for entry in walkdir::WalkDir::new(docs_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension() == Some("md".as_ref()) {
            let contents = std::fs::read_to_string(entry.path()).unwrap();
            
            // Check relative links
            let link_regex = regex::Regex::new(r"\]\(\.?\.?/([^)]+)\)").unwrap();
            for cap in link_regex.captures_iter(&contents) {
                let link = &cap[1];
                let link_path = entry.path().parent().unwrap().join(link);
                
                if !link_path.exists() && !link.starts_with("http") {
                    broken_links.push(format!(
                        "{}:{}: broken link to {}", 
                        entry.path().display(),
                        0, // Would need line number
                        link
                    ));
                }
            }
        }
    }
    
    assert!(
        broken_links.is_empty(),
        "Found broken links:\n{}", 
        broken_links.join("\n")
    );
}

/// Check all code examples compile
#[test]
fn check_code_examples() {
    let docs_dir = std::path::Path::new("docs");
    let mut errors = vec![];
    
    for entry in walkdir::WalkDir::new(docs_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension() == Some("md".as_ref()) {
            let contents = std::fs::read_to_string(entry.path()).unwrap();
            
            // Extract rust code blocks
            let code_block_regex = regex::Regex::new(r"```rust\n([\s\S]*?)```").unwrap();
            for cap in code_block_regex.captures_iter(&contents) {
                let code = &cap[1];
                
                // Skip if marked as no_run
                if code.contains("# // no_run") {
                    continue;
                }
                
                // Try to compile
                if let Err(e) = check_rust_syntax(code) {
                    errors.push(format!(
                        "{}: invalid Rust code: {}", 
                        entry.path().display(),
                        e
                    ));
                }
            }
        }
    }
    
    assert!(
        errors.is_empty(),
        "Found code errors:\n{}", 
        errors.join("\n")
    );
}

/// Check required documentation exists
#[test]
fn check_required_docs() {
    let required = [
        "docs/README.md",
        "docs/QUICKSTART.md",
        "docs/API.md",
        "docs/CLI.md",
        "docs/MCP-TOOLS.md",
    ];
    
    for path in required {
        assert!(
            std::path::Path::new(path).exists(),
            "Missing required documentation: {}", path
        );
    }
}

/// Check MCP tool documentation completeness
#[test]
fn check_mcp_tool_docs() {
    let mcp_docs = std::fs::read_to_string("docs/MCP-TOOLS.md").unwrap();
    let server = McpServer::new();
    let tools = server.tools();
    
    for tool in tools {
        assert!(
            mcp_docs.contains(&tool.name),
            "MCP tool '{}' not documented in MCP-TOOLS.md", tool.name
        );
    }
}
```

---

## 6. GitHub Actions Workflow

```yaml
# .github/workflows/docs-sync.yml

name: Documentation Sync

on:
  push:
    branches: [main, master]
    paths:
      - 'docs/**'
      - 'README.md'
      - 'CHANGELOG.md'
  
  release:
    types: [published]
  
  workflow_dispatch:

jobs:
  check-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Check documentation quality
        run: cargo test --test docs_quality
  
  sync-docs:
    needs: check-docs
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Checkout docs site
        uses: actions/checkout@v4
        with:
          repository: agentralabs/docs.agentic.so
          path: docs-site
          token: ${{ secrets.DOCS_DEPLOY_TOKEN }}
      
      - name: Sync documentation
        env:
          SISTER_NAME: ${{ env.SISTER_NAME }}
        run: |
          # Create target directory
          mkdir -p docs-site/content/${SISTER_NAME}
          
          # Copy documentation
          cp -r docs/* docs-site/content/${SISTER_NAME}/
          
          # Transform for Hugo
          # ... (transformation logic)
      
      - name: Commit and push
        run: |
          cd docs-site
          git config user.name "Agentic Bot"
          git config user.email "bot@agentic.so"
          
          git add content/${SISTER_NAME}
          
          if git diff --staged --quiet; then
            echo "No changes to commit"
          else
            git commit -m "docs(${SISTER_NAME}): sync from repository"
            git push
          fi
      
      - name: Trigger site rebuild
        uses: peter-evans/repository-dispatch@v2
        with:
          repository: agentralabs/docs.agentic.so
          event-type: rebuild-site
          token: ${{ secrets.DOCS_DEPLOY_TOKEN }}
```

---

## 7. Package Description Extraction

```bash
#!/usr/bin/env bash
# scripts/extract-description.sh
# Extract package description from README for registries

set -euo pipefail

README="README.md"
OUTPUT="${1:-/dev/stdout}"

# Extract first 3 paragraphs after the title
awk '
BEGIN { para=0; skip_title=1 }
/^#/ { if (skip_title) { skip_title=0; next } }
/^$/ { if (para > 0) para++ }
/^[^#]/ { if (!skip_title && para < 3) { print; para++ } }
para >= 3 { exit }
' "$README" > "$OUTPUT"
```

---

## 8. Documentation Templates

### QUICKSTART.md Template

```markdown
# Quickstart

Get started with Agentic {Sister} in under 5 minutes.

## Installation

### One-Line Install

```bash
curl -fsSL https://agentic.sh/{sister} | bash
```

### Using Cargo

```bash
cargo install agentic-{sister}-mcp
```

## Basic Usage

### CLI

```bash
# Quick status
agentic-{sister} status

# Main operation
agentic-{sister} {main-command}
```

### MCP Integration

Add to your Claude Desktop config:

```json
{
  "mcpServers": {
    "agentic-{sister}": {
      "command": "agentic-{sister}",
      "args": ["serve"]
    }
  }
}
```

Then restart Claude Desktop.

## Next Steps

- [Full API Reference](API.md)
- [CLI Commands](CLI.md)
- [MCP Tools](MCP-TOOLS.md)
```

### MCP-TOOLS.md Template

```markdown
# MCP Tools Reference

Agentic {Sister} provides {N} MCP tools for {description}.

## Tool Overview

| Tool | Description |
|------|-------------|
| `{sister}_core` | Core operations |
| `{sister}_workspace` | Workspace management |
| ... | ... |

## Tool Details

### {sister}_core

Core operations for {sister}.

**Operations:**
- `create` - Create a new {thing}
- `read` - Read {thing} details
- `update` - Update {thing}
- `delete` - Delete {thing}

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `operation` | string | Yes | Operation to perform |
| `id` | string | Depends | {Thing} ID |
| ... | ... | ... | ... |

**Example:**

```json
{
  "name": "{sister}_core",
  "arguments": {
    "operation": "create",
    "title": "Example"
  }
}
```

...
```

---

## 9. Summary

```
DOCUMENTATION REQUIREMENTS:
═══════════════════════════

□ docs/ folder is source of truth
□ All required docs exist (README, QUICKSTART, API, CLI, MCP-TOOLS)
□ All internal links valid
□ All code examples compile
□ All MCP tools documented
□ GitHub Action syncs to docs.agentic.so
□ Package descriptions extracted from README
□ Assets (diagrams, logos) included
□ CHANGELOG symlinked or included
```

---

*Document: SPEC-DOCS-PUBLIC-SYNC.md*
*Applies to: ALL Sisters*
