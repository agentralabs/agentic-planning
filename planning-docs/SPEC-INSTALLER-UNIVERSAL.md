# SPEC-INSTALLER-UNIVERSAL

> **Scope:** ALL Sisters
> **Status:** Canonical Standard
> **Enforced:** CI Guardrails

---

## 1. Overview

Every sister MUST provide a universal installer that works across:
- Desktop environments (Claude Desktop, Cursor, VS Code, Windsurf)
- Terminal environments (CLI usage)
- Server environments (remote deployment, authenticated)

The installer MUST follow the hardening patterns proven in AgenticMemory and AgenticCodebase.

---

## 2. Installation Profiles

```
PROFILE MATRIX:
═══════════════

┌────────────┬─────────────────────────────────────────────────────────────────┐
│ Profile    │ Description                                                     │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ desktop    │ MCP client config (Claude Desktop, Cursor, etc.)               │
│            │ - Configures mcpServers in appropriate config file             │
│            │ - Merge-only (never destructive overwrite)                     │
│            │ - Post-install restart guidance                                │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ terminal   │ CLI-only installation                                          │
│            │ - Binary to ~/.local/bin or /usr/local/bin                     │
│            │ - PATH configuration                                           │
│            │ - Shell completion                                             │
├────────────┼─────────────────────────────────────────────────────────────────┤
│ server     │ Remote/daemon deployment                                       │
│            │ - Token-based authentication required                          │
│            │ - Systemd/launchd service files                               │
│            │ - No interactive prompts                                       │
└────────────┴─────────────────────────────────────────────────────────────────┘
```

---

## 3. Installer Script Template

```bash
#!/usr/bin/env bash
# Universal installer for agentic-{sister}
# Usage: curl -fsSL https://agentic.sh/{sister} | bash

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================

SISTER_NAME="planning"  # Change per sister
BINARY_NAME="agentic-${SISTER_NAME}"
SHORT_NAME="aplan"      # Short alias (optional)
REPO="agentralabs/agentic-${SISTER_NAME}"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.agentic/${SISTER_NAME}"

# Version (empty = latest)
VERSION="${AGENTIC_VERSION:-}"

# Profile (desktop|terminal|server)
PROFILE="${AGENTIC_PROFILE:-desktop}"

# Non-interactive mode
NON_INTERACTIVE="${AGENTIC_NON_INTERACTIVE:-false}"

# ============================================================================
# COLORS AND OUTPUT
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
die() { error "$*"; exit 1; }

# ============================================================================
# PLATFORM DETECTION
# ============================================================================

detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Darwin) os="darwin" ;;
        Linux)  os="linux" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) die "Unsupported OS: $(uname -s)" ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) die "Unsupported architecture: $(uname -m)" ;;
    esac
    
    echo "${os}-${arch}"
}

# ============================================================================
# VERSION RESOLUTION
# ============================================================================

get_latest_version() {
    local latest
    latest=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | 
             grep '"tag_name"' | 
             sed -E 's/.*"v?([^"]+)".*/\1/')
    
    if [[ -z "$latest" ]]; then
        die "Failed to fetch latest version"
    fi
    
    echo "$latest"
}

# ============================================================================
# DOWNLOAD AND INSTALL
# ============================================================================

download_binary() {
    local platform="$1"
    local version="$2"
    local url="https://github.com/${REPO}/releases/download/v${version}/${BINARY_NAME}-${platform}.tar.gz"
    local tmp_dir
    
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT
    
    info "Downloading ${BINARY_NAME} v${version} for ${platform}..."
    
    if ! curl -fsSL "$url" -o "${tmp_dir}/archive.tar.gz"; then
        die "Download failed. Check if release exists: $url"
    fi
    
    tar -xzf "${tmp_dir}/archive.tar.gz" -C "$tmp_dir"
    
    # Install binary
    mkdir -p "$INSTALL_DIR"
    mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    # Create short alias if defined
    if [[ -n "$SHORT_NAME" ]]; then
        ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${SHORT_NAME}"
    fi
    
    success "Installed ${BINARY_NAME} to ${INSTALL_DIR}"
}

# ============================================================================
# PATH CONFIGURATION
# ============================================================================

ensure_path() {
    local shell_rc=""
    
    case "${SHELL:-}" in
        */zsh)  shell_rc="${HOME}/.zshrc" ;;
        */bash) shell_rc="${HOME}/.bashrc" ;;
        */fish) shell_rc="${HOME}/.config/fish/config.fish" ;;
        *)      shell_rc="${HOME}/.profile" ;;
    esac
    
    if [[ -f "$shell_rc" ]] && grep -q "${INSTALL_DIR}" "$shell_rc"; then
        return 0
    fi
    
    info "Adding ${INSTALL_DIR} to PATH..."
    
    if [[ "${SHELL:-}" == */fish ]]; then
        echo "fish_add_path ${INSTALL_DIR}" >> "$shell_rc"
    else
        echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$shell_rc"
    fi
    
    success "Added to PATH in $shell_rc"
    warn "Run 'source $shell_rc' or restart your shell"
}

# ============================================================================
# MCP CONFIGURATION (Desktop Profile)
# ============================================================================

configure_mcp_desktop() {
    local config_file=""
    local mcp_config=""
    
    # Detect MCP client
    case "$(uname -s)" in
        Darwin)
            # Check for Claude Desktop
            if [[ -d "${HOME}/Library/Application Support/Claude" ]]; then
                config_file="${HOME}/Library/Application Support/Claude/claude_desktop_config.json"
            fi
            # Check for Cursor
            if [[ -d "${HOME}/.cursor" ]]; then
                config_file="${HOME}/.cursor/mcp.json"
            fi
            ;;
        Linux)
            if [[ -d "${HOME}/.config/claude" ]]; then
                config_file="${HOME}/.config/claude/claude_desktop_config.json"
            fi
            ;;
    esac
    
    if [[ -z "$config_file" ]]; then
        warn "No MCP client detected. Skipping MCP configuration."
        return 0
    fi
    
    info "Configuring MCP client: $config_file"
    
    # Create config directory if needed
    mkdir -p "$(dirname "$config_file")"
    
    # MCP server configuration for this sister
    mcp_config=$(cat <<EOF
{
  "mcpServers": {
    "${BINARY_NAME}": {
      "command": "${INSTALL_DIR}/${BINARY_NAME}",
      "args": ["serve"],
      "env": {}
    }
  }
}
EOF
)
    
    # CRITICAL: Merge-only update (never destructive overwrite)
    if [[ -f "$config_file" ]]; then
        info "Merging with existing MCP configuration..."
        
        # Use jq for safe JSON merging if available
        if command -v jq &> /dev/null; then
            local existing
            existing=$(cat "$config_file")
            
            # Deep merge mcpServers
            echo "$existing" | jq --argjson new "$mcp_config" '
                .mcpServers = (.mcpServers // {}) + $new.mcpServers
            ' > "${config_file}.tmp"
            
            mv "${config_file}.tmp" "$config_file"
        else
            # Fallback: manual merge (less safe but works)
            warn "jq not found. Please manually add MCP configuration."
            echo ""
            echo "Add to $config_file:"
            echo "$mcp_config"
            return 0
        fi
    else
        echo "$mcp_config" > "$config_file"
    fi
    
    success "MCP configuration updated"
    
    # CRITICAL: Post-install restart guidance
    echo ""
    warn "╔════════════════════════════════════════════════════════════════╗"
    warn "║  IMPORTANT: Restart your MCP client to load the new server    ║"
    warn "║                                                                ║"
    warn "║  • Claude Desktop: Quit and reopen                            ║"
    warn "║  • Cursor: Reload window (Cmd/Ctrl+Shift+P → Reload)          ║"
    warn "║  • VS Code: Restart extension host                            ║"
    warn "╚════════════════════════════════════════════════════════════════╝"
}

# ============================================================================
# SERVER MODE CONFIGURATION
# ============================================================================

configure_server_mode() {
    info "Configuring server mode..."
    
    # Create config directory
    mkdir -p "$CONFIG_DIR"
    
    # Generate auth token if not provided
    if [[ -z "${AGENTIC_AUTH_TOKEN:-}" ]]; then
        local token
        token=$(openssl rand -hex 32)
        echo "$token" > "${CONFIG_DIR}/auth_token"
        chmod 600 "${CONFIG_DIR}/auth_token"
        
        info "Generated auth token: ${CONFIG_DIR}/auth_token"
        warn "Set AGENTIC_AUTH_TOKEN environment variable to use this token"
    fi
    
    # Install systemd service (Linux)
    if [[ "$(uname -s)" == "Linux" ]] && command -v systemctl &> /dev/null; then
        install_systemd_service
    fi
    
    # Install launchd service (macOS)
    if [[ "$(uname -s)" == "Darwin" ]]; then
        install_launchd_service
    fi
}

install_systemd_service() {
    local service_file="${HOME}/.config/systemd/user/${BINARY_NAME}.service"
    
    mkdir -p "$(dirname "$service_file")"
    
    cat > "$service_file" <<EOF
[Unit]
Description=Agentic ${SISTER_NAME^} MCP Server
After=network.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/${BINARY_NAME} serve --mode http
Restart=on-failure
RestartSec=5
Environment=AGENTIC_AUTH_MODE=required
EnvironmentFile=-${CONFIG_DIR}/env

# Resource limits
Nice=10
IOSchedulingClass=idle
MemoryMax=512M

[Install]
WantedBy=default.target
EOF
    
    systemctl --user daemon-reload
    systemctl --user enable "${BINARY_NAME}"
    
    success "Installed systemd service"
    info "Start with: systemctl --user start ${BINARY_NAME}"
}

install_launchd_service() {
    local plist_file="${HOME}/Library/LaunchAgents/com.agentic.${SISTER_NAME}.plist"
    
    mkdir -p "$(dirname "$plist_file")"
    
    cat > "$plist_file" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.agentic.${SISTER_NAME}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_DIR}/${BINARY_NAME}</string>
        <string>serve</string>
        <string>--mode</string>
        <string>http</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>StandardOutPath</key>
    <string>${CONFIG_DIR}/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>${CONFIG_DIR}/daemon.log</string>
    <key>LowPriorityIO</key>
    <true/>
    <key>Nice</key>
    <integer>10</integer>
</dict>
</plist>
EOF
    
    success "Installed launchd service"
    info "Start with: launchctl load $plist_file"
}

# ============================================================================
# DAEMON SETUP (Optional)
# ============================================================================

maybe_install_daemon() {
    # Skip if non-interactive or terminal profile
    if [[ "$NON_INTERACTIVE" == "true" ]] || [[ "$PROFILE" == "terminal" ]]; then
        return 0
    fi
    
    # Check if daemon is supported
    if ! "${INSTALL_DIR}/${BINARY_NAME}" daemon --help &> /dev/null 2>&1; then
        return 0
    fi
    
    echo ""
    read -rp "Install background daemon for continuous operation? [Y/n] " response
    response="${response:-Y}"
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        info "Installing daemon..."
        "${INSTALL_DIR}/${BINARY_NAME}" daemon install
        "${INSTALL_DIR}/${BINARY_NAME}" daemon start
        success "Daemon installed and started"
    fi
}

# ============================================================================
# OPTIONAL FEEDBACK
# ============================================================================

maybe_prompt_feedback() {
    if [[ "$NON_INTERACTIVE" == "true" ]]; then
        return 0
    fi
    
    echo ""
    info "Help improve ${BINARY_NAME}!"
    info "Report issues: https://github.com/${REPO}/issues"
    info "Documentation: https://docs.agentic.so/${SISTER_NAME}"
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║        Agentic ${SISTER_NAME^} Installer                              ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Platform: $platform"
    
    # Get version
    if [[ -z "$VERSION" ]]; then
        VERSION=$(get_latest_version)
    fi
    info "Version: $VERSION"
    info "Profile: $PROFILE"
    
    # Download and install
    download_binary "$platform" "$VERSION"
    
    # Configure PATH
    ensure_path
    
    # Profile-specific configuration
    case "$PROFILE" in
        desktop)
            configure_mcp_desktop
            maybe_install_daemon
            ;;
        terminal)
            # Just binary install, already done
            ;;
        server)
            configure_server_mode
            ;;
        *)
            warn "Unknown profile: $PROFILE. Using terminal profile."
            ;;
    esac
    
    # Optional feedback
    maybe_prompt_feedback
    
    echo ""
    success "Installation complete!"
    echo ""
    echo "Usage:"
    echo "  ${BINARY_NAME} --help"
    if [[ -n "$SHORT_NAME" ]]; then
        echo "  ${SHORT_NAME} --help"
    fi
    echo ""
}

main "$@"
```

---

## 4. Verification Script

```bash
#!/usr/bin/env bash
# scripts/check-install-commands.sh
# Verify installer script contains all required elements

set -euo pipefail

SCRIPT="scripts/install.sh"
REQUIRED_FUNCTIONS=(
    "detect_platform"
    "download_binary"
    "ensure_path"
    "configure_mcp_desktop"
    "configure_server_mode"
    "maybe_install_daemon"
)

REQUIRED_PATTERNS=(
    "PROFILE.*desktop|terminal|server"
    "merge.*MCP.*configuration"
    "Restart.*MCP.*client"
    "AGENTIC_AUTH"
)

errors=0

echo "Checking installer script: $SCRIPT"

# Check required functions
for func in "${REQUIRED_FUNCTIONS[@]}"; do
    if ! grep -q "^${func}()" "$SCRIPT"; then
        echo "MISSING: function $func"
        ((errors++))
    fi
done

# Check required patterns
for pattern in "${REQUIRED_PATTERNS[@]}"; do
    if ! grep -Eq "$pattern" "$SCRIPT"; then
        echo "MISSING: pattern $pattern"
        ((errors++))
    fi
done

# Check for forbidden patterns
if grep -q "rm -rf.*config" "$SCRIPT"; then
    echo "FORBIDDEN: destructive config removal"
    ((errors++))
fi

if grep -q "> .*config.*json" "$SCRIPT" && ! grep -q "merge" "$SCRIPT"; then
    echo "WARNING: potential config overwrite without merge"
    ((errors++))
fi

if [[ $errors -gt 0 ]]; then
    echo ""
    echo "FAILED: $errors issues found"
    exit 1
else
    echo "PASSED: All checks passed"
fi
```

---

## 5. Environment Variables

```
INSTALLER ENVIRONMENT VARIABLES:
════════════════════════════════

AGENTIC_VERSION        - Specific version to install (default: latest)
AGENTIC_PROFILE        - Installation profile: desktop|terminal|server
AGENTIC_NON_INTERACTIVE - Skip all prompts (default: false)
AGENTIC_AUTH_TOKEN     - Pre-configured auth token for server mode
AGENTIC_INSTALL_DIR    - Custom installation directory
```

---

## 6. Testing Matrix

```
REQUIRED TESTS:
═══════════════

✓ Fresh install on clean system
✓ Upgrade from previous version
✓ Multiple profile installations
✓ MCP config merge (not overwrite)
✓ Post-install restart guidance displayed
✓ Daemon installation optional prompt
✓ Server mode auth token generation
✓ Platform detection (macOS arm64, macOS x64, Linux x64)
✓ PATH configuration (bash, zsh, fish)
✓ Non-interactive mode
```

---

*Document: SPEC-INSTALLER-UNIVERSAL.md*
*Applies to: ALL Sisters*
