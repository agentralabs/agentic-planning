#!/usr/bin/env bash
# ============================================================================
# agentic-planning installer
# Universal installer with profile support, MCP auto-configuration, and
# optional pre-built binary download.
#
# Usage:
#   curl -sSf https://raw.githubusercontent.com/agentra/agentic-planning/main/scripts/install.sh | bash
#   bash scripts/install.sh --profile server --dir /opt/agentic
#
# Profiles:
#   desktop   (default) — MCP server + CLI + auto-configure MCP clients
#   terminal  — same as desktop, optimised for headless / terminal-only usage
#   server    — binary only, auth enforced, no MCP client config
# ============================================================================
set -euo pipefail

# ── Defaults ────────────────────────────────────────────────────────────
VERSION="${AGENTIC_VERSION:-latest}"
PROFILE="${AGENTIC_PROFILE:-desktop}"
INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"
DRY_RUN="${DRY_RUN:-false}"
REPO_OWNER="agentra"
REPO_NAME="agentic-planning"
BINARY_MCP="agentic-planning-mcp"
BINARY_CLI="aplan"
DATA_DIR="${HOME}/.agentic-planning"
SERVER_ARGS_TEXT='["--mode", "stdio"]'

# ── Colour / UI helpers ────────────────────────────────────────────────
green()  { printf "\033[0;32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[0;33m%s\033[0m\n" "$*"; }
red()    { printf "\033[0;31m%s\033[0m\n" "$*"; }
bold()   { printf "\033[1m%s\033[0m\n" "$*"; }
dim()    { printf "\033[2m%s\033[0m\n" "$*"; }

progress_bar() {
  local current=$1 total=$2 width=40
  local pct=$((current * 100 / total))
  local filled=$((current * width / total))
  local empty=$((width - filled))
  local bar=""
  for ((i=0; i<filled; i++)); do bar+="█"; done
  for ((i=0; i<empty;  i++)); do bar+="░"; done
  printf "\r  [%s] %3d%%" "$bar" "$pct"
}

step_count=0
total_steps=6

step() {
  step_count=$((step_count + 1))
  echo ""
  bold "[$step_count/$total_steps] $1"
}

# ── Argument parsing ──────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)  VERSION="$2"; shift 2 ;;
    --profile)  PROFILE="$2"; shift 2 ;;
    --dir)      INSTALL_DIR="$2"; shift 2 ;;
    --dry-run)  DRY_RUN=true; shift ;;
    --help|-h)
      echo "Usage: install.sh [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --version <ver>   Version to install (default: latest)"
      echo "  --profile <p>     desktop|terminal|server (default: desktop)"
      echo "  --dir <path>      Installation directory (default: ~/.local/bin)"
      echo "  --dry-run         Show what would be done without doing it"
      echo "  -h, --help        Show this help"
      exit 0
      ;;
    *) red "Unknown option: $1"; exit 1 ;;
  esac
done

# Validate profile
case "$PROFILE" in
  desktop|terminal|server) ;;
  *) red "Invalid profile: $PROFILE (must be desktop, terminal, or server)"; exit 1 ;;
esac

# ── Banner ─────────────────────────────────────────────────────────────
echo ""
bold "╔══════════════════════════════════════════════════════════╗"
bold "║          agentic-planning installer                     ║"
bold "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Version:   ${VERSION}"
echo "  Profile:   ${PROFILE}"
echo "  Directory: ${INSTALL_DIR}"
echo "  Dry run:   ${DRY_RUN}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
  yellow "DRY RUN — no changes will be made."
fi

# ── Step 1: Platform detection ─────────────────────────────────────────
step "Detecting platform..."

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  darwin)  OS_LABEL="macOS" ;;
  linux)   OS_LABEL="Linux" ;;
  mingw*|msys*|cygwin*) OS="windows"; OS_LABEL="Windows" ;;
  *) red "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) red "Unsupported architecture: $ARCH"; exit 1 ;;
esac

PLATFORM="${OS}-${ARCH}"
green "  Platform: ${OS_LABEL} ${ARCH} (${PLATFORM})"

# ── Step 2: Check dependencies ─────────────────────────────────────────
step "Checking dependencies..."

HAS_CARGO=false
HAS_CURL=false
HAS_JQ=false
HAS_PYTHON3=false

command -v cargo   &>/dev/null && HAS_CARGO=true
command -v curl    &>/dev/null && HAS_CURL=true
command -v jq      &>/dev/null && HAS_JQ=true
command -v python3 &>/dev/null && HAS_PYTHON3=true

[[ "$HAS_CARGO" == "true" ]]   && green "  ✓ cargo"   || yellow "  ✗ cargo (needed for source build)"
[[ "$HAS_CURL" == "true" ]]    && green "  ✓ curl"    || yellow "  ✗ curl (needed for binary download)"
[[ "$HAS_JQ" == "true" ]]      && green "  ✓ jq"      || dim   "  ○ jq (optional, for MCP config)"
[[ "$HAS_PYTHON3" == "true" ]] && green "  ✓ python3" || dim   "  ○ python3 (jq fallback)"

# Need either curl (for download) or cargo (for source build)
if [[ "$HAS_CURL" == "false" && "$HAS_CARGO" == "false" ]]; then
  red "Error: need either curl (for binary download) or cargo (for source build)."
  red "  Install Rust: https://rustup.rs"
  exit 1
fi

# ── Step 3: Install binaries ───────────────────────────────────────────
step "Installing binaries..."

mkdir -p "${INSTALL_DIR}"
mkdir -p "${DATA_DIR}"

INSTALLED=()
BUILD_FROM_SOURCE=false

# Try pre-built binary download first (if curl available and version is known)
try_download() {
  local bin_name="$1"
  local url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/v${VERSION}/${bin_name}-${PLATFORM}"

  if [[ "$VERSION" == "latest" ]]; then
    # Try to resolve latest tag
    local latest_url="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    local tag
    tag=$(curl -sSf "$latest_url" 2>/dev/null | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/' || echo "")
    if [[ -n "$tag" ]]; then
      VERSION="$tag"
      url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/v${VERSION}/${bin_name}-${PLATFORM}"
    else
      return 1
    fi
  fi

  if [[ "$DRY_RUN" == "true" ]]; then
    yellow "  (dry run) Would download: ${url}"
    return 0
  fi

  local dest="${INSTALL_DIR}/${bin_name}"
  if curl -sSfL -o "${dest}" "$url" 2>/dev/null; then
    chmod +x "${dest}"
    return 0
  fi
  return 1
}

# Attempt binary download
if [[ "$HAS_CURL" == "true" ]]; then
  dim "  Attempting pre-built binary download..."
  if try_download "$BINARY_MCP" && try_download "$BINARY_CLI"; then
    INSTALLED+=("$BINARY_MCP" "$BINARY_CLI")
    green "  ✓ Downloaded pre-built binaries"
  else
    yellow "  Pre-built binaries not available, falling back to source build..."
    BUILD_FROM_SOURCE=true
  fi
else
  BUILD_FROM_SOURCE=true
fi

# Source build fallback
if [[ "$BUILD_FROM_SOURCE" == "true" ]]; then
  if [[ "$HAS_CARGO" == "false" ]]; then
    red "Error: cargo required for source build. Install Rust: https://rustup.rs"
    exit 1
  fi

  # Detect if we're inside the repo
  SCRIPT_DIR=""
  if [[ -f "$(cd "$(dirname "$0")/.." 2>/dev/null && pwd)/Cargo.toml" ]]; then
    SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
  fi

  if [[ -n "$SCRIPT_DIR" ]]; then
    dim "  Building from local source: ${SCRIPT_DIR}"
    if [[ "$DRY_RUN" == "true" ]]; then
      yellow "  (dry run) Would build from source"
    else
      progress_bar 1 4
      cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" \
        -p agentic-planning-mcp \
        -p agentic-planning-cli 2>&1 | while IFS= read -r _line; do :; done
      progress_bar 3 4

      # Copy binaries
      if [[ -f "${SCRIPT_DIR}/target/release/${BINARY_MCP}" ]]; then
        cp "${SCRIPT_DIR}/target/release/${BINARY_MCP}" "${INSTALL_DIR}/"
        INSTALLED+=("$BINARY_MCP")
      fi
      if [[ -f "${SCRIPT_DIR}/target/release/${BINARY_CLI}" ]]; then
        cp "${SCRIPT_DIR}/target/release/${BINARY_CLI}" "${INSTALL_DIR}/"
        INSTALLED+=("$BINARY_CLI")
      elif [[ -f "${SCRIPT_DIR}/target/release/agentic-planning-cli" ]]; then
        cp "${SCRIPT_DIR}/target/release/agentic-planning-cli" "${INSTALL_DIR}/${BINARY_CLI}"
        INSTALLED+=("$BINARY_CLI")
      fi
      progress_bar 4 4
      echo ""
    fi
  else
    dim "  Building from crates.io (cargo install)..."
    if [[ "$DRY_RUN" == "true" ]]; then
      yellow "  (dry run) Would cargo install agentic-planning-mcp agentic-planning-cli"
    else
      cargo install agentic-planning-mcp --root "${INSTALL_DIR%/bin}" 2>/dev/null && INSTALLED+=("$BINARY_MCP") || true
      cargo install agentic-planning-cli --root "${INSTALL_DIR%/bin}" 2>/dev/null && INSTALLED+=("$BINARY_CLI") || true
    fi
  fi
fi

if [[ "$DRY_RUN" == "false" && ${#INSTALLED[@]} -eq 0 ]]; then
  red "Error: no binaries installed."
  exit 1
fi

green "  ✓ Binaries installed to ${INSTALL_DIR}"

# ── Step 4: Create MCP launcher entrypoint ─────────────────────────────
step "Creating MCP launcher..."

LAUNCHER="${INSTALL_DIR}/agentic-planning-mcp-launcher"

if [[ "$DRY_RUN" == "true" ]]; then
  yellow "  (dry run) Would create launcher: ${LAUNCHER}"
else
  cat > "${LAUNCHER}" << 'LAUNCHER_SCRIPT'
#!/usr/bin/env bash
# MCP launcher for agentic-planning
# Resolves project-aware data directory and launches the MCP server.
set -euo pipefail

BINARY_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="${BINARY_DIR}/agentic-planning-mcp"

# Project-aware data directory
if [[ -n "${AGENTIC_DATA_DIR:-}" ]]; then
  DATA_DIR="$AGENTIC_DATA_DIR"
elif [[ -d ".git" ]]; then
  PROJECT_NAME="$(basename "$(pwd)")"
  DATA_DIR="${HOME}/.agentic-planning/projects/${PROJECT_NAME}"
else
  DATA_DIR="${HOME}/.agentic-planning"
fi

mkdir -p "${DATA_DIR}"
export AGENTIC_DATA_DIR="${DATA_DIR}"

exec "${BINARY}" "$@"
LAUNCHER_SCRIPT
  chmod +x "${LAUNCHER}"
  green "  ✓ Launcher created: ${LAUNCHER}"
fi

# ── Step 5: MCP client auto-configuration ──────────────────────────────
step "Configuring MCP clients..."

if [[ "$PROFILE" == "server" ]]; then
  yellow "  Server profile: skipping MCP client configuration."
  yellow "  Server auth will be enforced (set AGENTIC_TOKEN)."
else
  # MCP server config entry to merge
  MCP_ENTRY='{
    "agentic-planning": {
      "command": "'"${LAUNCHER}"'",
      "args": ["--mode", "stdio"]
    }
  }'

  # JSON merge helper — tries jq first, falls back to python3
  merge_json_config() {
    local config_file="$1"
    local key_path="$2" # e.g., "mcpServers"

    if [[ ! -f "$config_file" ]]; then
      # Create new config with just the MCP entry
      if [[ "$HAS_JQ" == "true" ]]; then
        echo "{}" | jq --argjson entry "$MCP_ENTRY" ".${key_path} = (\".${key_path}\" // {}) * \$entry" > "${config_file}.tmp"
      elif [[ "$HAS_PYTHON3" == "true" ]]; then
        python3 -c "
import json, sys
entry = json.loads('''$MCP_ENTRY''')
doc = {'${key_path}': entry}
with open('${config_file}.tmp', 'w') as f:
    json.dump(doc, f, indent=2)
"
      else
        yellow "    (no jq or python3 — manual config required)"
        return 1
      fi
      mv "${config_file}.tmp" "$config_file"
      return 0
    fi

    # Merge into existing config
    if [[ "$HAS_JQ" == "true" ]]; then
      jq --argjson entry "$MCP_ENTRY" \
        ".${key_path} = ((.${key_path} // {}) * \$entry)" \
        "$config_file" > "${config_file}.tmp" && mv "${config_file}.tmp" "$config_file"
    elif [[ "$HAS_PYTHON3" == "true" ]]; then
      python3 -c "
import json, sys
entry = json.loads('''$MCP_ENTRY''')
with open('$config_file', 'r') as f:
    doc = json.load(f)
section = doc.get('$key_path', {})
section.update(entry)
doc['$key_path'] = section
with open('${config_file}.tmp', 'w') as f:
    json.dump(doc, f, indent=2)
" && mv "${config_file}.tmp" "$config_file"
    else
      yellow "    (no jq or python3 — manual config required)"
      return 1
    fi
  }

  configure_client() {
    local name="$1"
    local config_file="$2"
    local key_path="$3"
    local config_dir
    config_dir="$(dirname "$config_file")"

    if [[ "$DRY_RUN" == "true" ]]; then
      yellow "  (dry run) Would configure ${name}: ${config_file}"
      return
    fi

    # Only configure if the config directory exists (client is installed)
    if [[ ! -d "$config_dir" ]]; then
      return
    fi

    # Backup existing config
    if [[ -f "$config_file" ]]; then
      cp "$config_file" "${config_file}.bak.$(date +%s)"
    fi

    mkdir -p "$config_dir"
    if merge_json_config "$config_file" "$key_path"; then
      green "  ✓ ${name}"
    else
      yellow "  ✗ ${name} (manual configuration needed)"
    fi
  }

  CONFIGURED=0

  # Claude Desktop
  if [[ "$OS" == "darwin" ]]; then
    CLAUDE_DESKTOP="${HOME}/Library/Application Support/Claude/claude_desktop_config.json"
  else
    CLAUDE_DESKTOP="${HOME}/.config/Claude/claude_desktop_config.json"
  fi
  configure_client "Claude Desktop" "$CLAUDE_DESKTOP" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true

  # Claude Code (claude.ai CLI)
  CLAUDE_CODE="${HOME}/.claude/settings.json"
  if [[ -d "${HOME}/.claude" ]]; then
    configure_client "Claude Code" "$CLAUDE_CODE" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true
  fi

  # Cursor
  if [[ "$OS" == "darwin" ]]; then
    CURSOR_CONFIG="${HOME}/Library/Application Support/Cursor/User/globalStorage/cursor.mcp/config.json"
  else
    CURSOR_CONFIG="${HOME}/.config/Cursor/User/globalStorage/cursor.mcp/config.json"
  fi
  configure_client "Cursor" "$CURSOR_CONFIG" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true

  # Windsurf
  if [[ "$OS" == "darwin" ]]; then
    WINDSURF_CONFIG="${HOME}/Library/Application Support/Windsurf/User/globalStorage/windsurf.mcp/config.json"
  else
    WINDSURF_CONFIG="${HOME}/.config/Windsurf/User/globalStorage/windsurf.mcp/config.json"
  fi
  configure_client "Windsurf" "$WINDSURF_CONFIG" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true

  # VS Code
  if [[ "$OS" == "darwin" ]]; then
    VSCODE_CONFIG="${HOME}/Library/Application Support/Code/User/globalStorage/vscode.mcp/config.json"
  else
    VSCODE_CONFIG="${HOME}/.config/Code/User/globalStorage/vscode.mcp/config.json"
  fi
  configure_client "VS Code" "$VSCODE_CONFIG" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true

  # VS Code Insiders
  if [[ "$OS" == "darwin" ]]; then
    VSCODE_INS="${HOME}/Library/Application Support/Code - Insiders/User/globalStorage/vscode.mcp/config.json"
  else
    VSCODE_INS="${HOME}/.config/Code - Insiders/User/globalStorage/vscode.mcp/config.json"
  fi
  configure_client "VS Code Insiders" "$VSCODE_INS" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true

  # VSCodium
  if [[ "$OS" == "darwin" ]]; then
    VSCODIUM="${HOME}/Library/Application Support/VSCodium/User/globalStorage/vscode.mcp/config.json"
  else
    VSCODIUM="${HOME}/.config/VSCodium/User/globalStorage/vscode.mcp/config.json"
  fi
  configure_client "VSCodium" "$VSCODIUM" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true

  # Codex CLI
  CODEX_CONFIG="${HOME}/.codex/config.json"
  if [[ -d "${HOME}/.codex" ]]; then
    configure_client "Codex CLI" "$CODEX_CONFIG" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true
  fi

  # Cline
  CLINE_CONFIG="${HOME}/.cline/mcp_settings.json"
  if [[ -d "${HOME}/.cline" ]]; then
    configure_client "Cline" "$CLINE_CONFIG" "mcpServers" && CONFIGURED=$((CONFIGURED + 1)) || true
  fi

  if [[ "$DRY_RUN" == "false" ]]; then
    dim "  Configured ${CONFIGURED} MCP client(s)"
  fi
fi

# ── Step 6: Profile-specific setup ─────────────────────────────────────
step "Applying profile: ${PROFILE}..."

case "$PROFILE" in
  desktop)
    green "  ✓ Desktop profile: MCP server + CLI + client auto-config"
    ;;
  terminal)
    green "  ✓ Terminal profile: MCP server + CLI + client auto-config"
    dim "  Tip: use 'aplan' for terminal access"
    ;;
  server)
    if [[ "$DRY_RUN" == "false" ]]; then
      # Create server env file with auth enforced
      SERVER_ENV="${DATA_DIR}/server.env"
      cat > "$SERVER_ENV" << 'EOF'
# agentic-planning server configuration
# Auth is required in server profile
AGENTIC_AUTH_MODE=required
# AGENTIC_TOKEN=<set-your-secret-here>
EOF
      green "  ✓ Server profile: auth enforced"
      yellow "  Set your auth token in: ${SERVER_ENV}"
    else
      yellow "  (dry run) Would create server.env with auth enforced"
    fi

    # Offer daemon installation
    if [[ "$DRY_RUN" == "false" && -t 0 ]]; then
      echo ""
      read -rp "  Install as systemd service? [y/N] " DAEMON_ANSWER
      if [[ "${DAEMON_ANSWER,,}" == "y" ]]; then
        if command -v systemctl &>/dev/null; then
          SERVICE_FILE="${HOME}/.config/systemd/user/agentic-planning.service"
          mkdir -p "$(dirname "$SERVICE_FILE")"
          cat > "$SERVICE_FILE" << SYSTEMD_EOF
[Unit]
Description=Agentic Planning MCP Server
After=network.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/${BINARY_MCP} --mode stdio
EnvironmentFile=${DATA_DIR}/server.env
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
SYSTEMD_EOF
          systemctl --user daemon-reload
          systemctl --user enable agentic-planning
          green "  ✓ Systemd service installed (start with: systemctl --user start agentic-planning)"
        else
          yellow "  systemd not available — manual daemon setup required"
        fi
      fi
    fi
    ;;
esac

# ── PATH check ─────────────────────────────────────────────────────────
if ! echo "${PATH}" | tr ':' '\n' | grep -qx "${INSTALL_DIR}"; then
  echo ""
  yellow "Note: ${INSTALL_DIR} is not in your PATH."
  yellow "Add to your shell profile:"
  echo "  export PATH=\"${INSTALL_DIR}:\${PATH}\""
fi

# ── Summary ────────────────────────────────────────────────────────────
echo ""
bold "╔══════════════════════════════════════════════════════════╗"
bold "║          Installation complete                          ║"
bold "╚══════════════════════════════════════════════════════════╝"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
  yellow "Dry run complete — no changes were made."
  echo "  Re-run without --dry-run to install."
else
  green "Binaries:"
  for b in "${INSTALLED[@]}"; do
    echo "  ${INSTALL_DIR}/${b}"
  done

  echo ""
  echo "MCP client summary:"
  echo "  Universal MCP entry (works in any MCP client):"
  echo "  command: ${LAUNCHER}"
  echo "  args: ${SERVER_ARGS_TEXT}"
  echo ""
  echo "Quick terminal check:"
  echo "  aplan version"

  echo ""
  echo "Next steps:"
  case "$PROFILE" in
    desktop|terminal)
      echo "  1. Restart your MCP client (Claude Desktop, Cursor, etc.)"
      echo "  2. Verify planning tools appear in the tool list"
      echo "  3. Try: aplan version"
      echo "  4. After restart, confirm 'agentic-planning' appears in your MCP server list."
      echo "  5. Optional feedback: open https://github.com/agentralabs/agentic-planning/issues"
      ;;
    server)
      echo "  1. Set your auth token:"
      echo "     export AGENTIC_TOKEN=your-secret"
      echo "  2. Start the server:"
      echo "     systemctl --user start agentic-planning"
      echo "  3. Or run manually:"
      echo "     AGENTIC_TOKEN=your-secret ${BINARY_MCP} --mode stdio"
      echo "  4. After restart, confirm 'agentic-planning' appears in your MCP server list."
      echo "  5. Optional feedback: open https://github.com/agentralabs/agentic-planning/issues"
      ;;
  esac
fi

echo ""
