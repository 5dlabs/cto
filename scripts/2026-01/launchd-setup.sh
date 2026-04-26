#!/bin/bash
# launchd-setup.sh - Generate and manage launchd services for CTO development
# =============================================================================
# This script generates launchd plist files with correct absolute paths and
# manages loading/unloading services.
#
# Usage:
#   ./scripts/2026-01/launchd-setup.sh install   # Generate and load all services
#   ./scripts/2026-01/launchd-setup.sh uninstall # Unload and remove all services
#   ./scripts/2026-01/launchd-setup.sh status    # Show service status
#   ./scripts/2026-01/launchd-setup.sh logs      # Tail all service logs
#   ./scripts/2026-01/launchd-setup.sh restart   # Restart all services
# =============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
LOG_DIR="/tmp/cto-launchd"
PLIST_PREFIX="ai.5dlabs.cto"

# Service definitions: name:binary:port
# Port 0 means no health check endpoint
# Binary "cloudflared" is handled specially
SERVICES=(
    "controller:agent-controller:8080"
    "pm-server:pm-server:8081"
    "healer:healer:8082"
    "healer-sensor:healer:0"
    "tunnel:cloudflared:0"
)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Generate environment variables from .env.local
generate_env_dict() {
    local env_file="$PROJECT_DIR/.env.local"
    
    # Start with required env vars
    cat << EOF
        <key>RUST_LOG</key>
        <string>info</string>
        <key>AGENT_TEMPLATES_PATH</key>
        <string>$PROJECT_DIR/templates</string>
        <key>HEALER_TEMPLATES_DIR</key>
        <string>$PROJECT_DIR/templates/healer</string>
        <key>CTO_CONFIG_PATH</key>
        <string>$PROJECT_DIR/cto-config.json</string>
        <key>CONTROLLER_CONFIG_PATH</key>
        <string>$PROJECT_DIR/config/controller-config.yaml</string>
        <key>SERVER_HOST</key>
        <string>0.0.0.0</string>
        <key>PATH</key>
        <string>$PATH</string>
EOF
    
    # Add secrets from .env.local if it exists
    if [[ -f "$env_file" ]]; then
        # Extract key env vars (be careful not to expose in logs)
        while IFS='=' read -r key value; do
            # Skip comments and empty lines
            [[ -z "$key" || "$key" =~ ^# ]] && continue
            # Handle 'export KEY=VALUE' format by stripping 'export '
            key="${key#export }"
            # Only include specific keys we need
            case "$key" in
                LINEAR_OAUTH_TOKEN|LINEAR_API_KEY|LINEAR_WEBHOOK_SECRET|LINEAR_ENABLED|LINEAR_WEBHOOK_URL|WEBHOOK_CALLBACK_URL|ANTHROPIC_API_KEY|GITHUB_TOKEN|OPENAI_API_KEY|KUBECONFIG)
                    # Remove quotes from value
                    value="${value%\"}"
                    value="${value#\"}"
                    value="${value%\'}"
                    value="${value#\'}"
                    echo "        <key>$key</key>"
                    echo "        <string>$value</string>"
                    ;;
                LINEAR_APP_*)
                    value="${value%\"}"
                    value="${value#\"}"
                    value="${value%\'}"
                    value="${value#\'}"
                    echo "        <key>$key</key>"
                    echo "        <string>$value</string>"
                    ;;
            esac
        done < "$env_file"
    fi
}

# Generate plist for a service
generate_service_plist() {
    local name="$1"
    local binary="$2"
    local port="$3"
    local label="$PLIST_PREFIX.$name"
    local binary_path="$PROJECT_DIR/target/release/$binary"
    
    # Special handling for different services
    local program_args
    if [[ "$name" == "healer" ]]; then
        program_args="        <string>$binary_path</string>
        <string>server</string>
        <string>--addr</string>
        <string>0.0.0.0:$port</string>"
    elif [[ "$name" == "healer-sensor" ]]; then
        program_args="        <string>$binary_path</string>
        <string>sensor</string>
        <string>github-actions</string>
        <string>--repositories=5dlabs/cto</string>
        <string>--poll-interval=300</string>
        <string>--lookback-mins=60</string>
        <string>--create-issues</string>
        <string>--issue-labels=healer,ci-failure</string>
        <string>--max-per-poll=3</string>
        <string>--namespace=cto</string>
        <string>--verbose</string>"
    elif [[ "$name" == "tunnel" ]]; then
        # Cloudflare tunnel - uses system cloudflared binary
        local cloudflared_path
        cloudflared_path=$(which cloudflared 2>/dev/null || echo "/opt/homebrew/bin/cloudflared")
        program_args="        <string>$cloudflared_path</string>
        <string>tunnel</string>
        <string>--config</string>
        <string>$PROJECT_DIR/config/cloudflared-pm-dev.yaml</string>
        <string>run</string>"
        # Override binary_path for the plist
        binary_path="$cloudflared_path"
    else
        program_args="        <string>$binary_path</string>"
    fi
    
    # Add port-specific env var
    local port_env=""
    if [[ "$name" == "controller" ]]; then
        port_env="        <key>SERVER_PORT</key>
        <string>$port</string>"
    fi
    
    cat << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$label</string>
    
    <key>ProgramArguments</key>
    <array>
$program_args
    </array>
    
    <key>WorkingDirectory</key>
    <string>$PROJECT_DIR</string>
    
    <key>EnvironmentVariables</key>
    <dict>
$(generate_env_dict)
$port_env
    </dict>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>RunAtLoad</key>
    <false/>
    
    <key>StandardOutPath</key>
    <string>$LOG_DIR/$name.log</string>
    
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/$name.err</string>
    
    <key>ThrottleInterval</key>
    <integer>5</integer>
</dict>
</plist>
EOF
}

# Generate plist for the watcher service
generate_watcher_plist() {
    local label="$PLIST_PREFIX.watcher"
    local watcher_script="$PROJECT_DIR/scripts/2026-01/cto-watcher.sh"
    
    cat << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$label</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>$watcher_script</string>
    </array>
    
    <key>WorkingDirectory</key>
    <string>$PROJECT_DIR</string>
    
    <key>EnvironmentVariables</key>
    <dict>
        <key>CTO_WATCH_DIR</key>
        <string>$PROJECT_DIR/target/release</string>
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>RunAtLoad</key>
    <false/>
    
    <key>StandardOutPath</key>
    <string>$LOG_DIR/watcher.log</string>
    
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/watcher.err</string>
</dict>
</plist>
EOF
}

# Install all services
install_services() {
    log_info "Installing CTO launchd services..."
    
    # Check prerequisites
    if ! command -v fswatch &> /dev/null; then
        log_error "fswatch is required but not installed. Run: brew install fswatch"
        exit 1
    fi
    
    # Check if binaries exist
    local missing_binaries=()
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        # Skip cloudflared - it's a system binary, not built from this project
        if [[ "$binary" == "cloudflared" ]]; then
            if ! command -v cloudflared &>/dev/null; then
                log_error "cloudflared not found. Install with: brew install cloudflared"
                exit 1
            fi
            continue
        fi
        if [[ ! -f "$PROJECT_DIR/target/release/$binary" ]]; then
            missing_binaries+=("$binary")
        fi
    done
    
    if [[ ${#missing_binaries[@]} -gt 0 ]]; then
        log_warn "Missing binaries: ${missing_binaries[*]}"
        log_warn "Run 'cargo build --release' first to build them."
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Create LaunchAgents directory if needed
    mkdir -p "$LAUNCH_AGENTS_DIR"
    
    # Generate and install service plists
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        local plist_file="$LAUNCH_AGENTS_DIR/$PLIST_PREFIX.$name.plist"
        
        log_info "Installing $name service..."
        generate_service_plist "$name" "$binary" "$port" > "$plist_file"
        
        # Unload if already loaded, then load
        launchctl unload "$plist_file" 2>/dev/null || true
        launchctl load "$plist_file"
    done
    
    # Generate and install watcher plist
    local watcher_plist="$LAUNCH_AGENTS_DIR/$PLIST_PREFIX.watcher.plist"
    log_info "Installing watcher service..."
    generate_watcher_plist > "$watcher_plist"
    launchctl unload "$watcher_plist" 2>/dev/null || true
    launchctl load "$watcher_plist"
    
    log_info "✅ All services installed!"
    log_info ""
    log_info "Services will auto-restart when you rebuild binaries with:"
    log_info "  cargo build --release"
    log_info ""
    log_info "View logs with:"
    log_info "  just launchd-logs"
    log_info ""
    log_info "Check status with:"
    log_info "  just launchd-status"
}

# Uninstall all services
uninstall_services() {
    log_info "Uninstalling CTO launchd services..."
    
    # Unload and remove service plists
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        local plist_file="$LAUNCH_AGENTS_DIR/$PLIST_PREFIX.$name.plist"
        
        if [[ -f "$plist_file" ]]; then
            log_info "Removing $name service..."
            launchctl unload "$plist_file" 2>/dev/null || true
            rm -f "$plist_file"
        fi
    done
    
    # Unload and remove watcher
    local watcher_plist="$LAUNCH_AGENTS_DIR/$PLIST_PREFIX.watcher.plist"
    if [[ -f "$watcher_plist" ]]; then
        log_info "Removing watcher service..."
        launchctl unload "$watcher_plist" 2>/dev/null || true
        rm -f "$watcher_plist"
    fi
    
    log_info "✅ All services uninstalled!"
}

# Show service status
show_status() {
    echo "═══ CTO launchd Service Status ═══"
    echo ""
    printf "%-25s %-10s %-8s %s\n" "SERVICE" "STATUS" "PID" "HEALTH"
    echo "────────────────────────────────────────────────────────────────"
    
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        local label="$PLIST_PREFIX.$name"
        
        # Get launchctl status
        local status pid health
        if launchctl list "$label" &>/dev/null; then
            pid=$(launchctl list "$label" 2>/dev/null | grep -E "^\s*\"PID\"" | awk '{print $3}' | tr -d '";' || echo "-")
            if [[ -z "$pid" || "$pid" == "-" ]]; then
                # Try alternative method
                pid=$(launchctl list | grep "$label" | awk '{print $1}')
                [[ "$pid" == "-" ]] && pid=""
            fi
            
            if [[ -n "$pid" && "$pid" != "-" ]]; then
                status="${GREEN}running${NC}"
                # Check health endpoint (skip if port is 0)
                if [[ "$name" == "tunnel" ]]; then
                    # Check if tunnel is actually working by hitting the dev URL
                    if curl -sf "https://pm-dev.5dlabs.ai/health" --max-time 5 &>/dev/null; then
                        health="${GREEN}✓ connected${NC}"
                    else
                        health="${YELLOW}? check tunnel${NC}"
                    fi
                elif [[ "$port" == "0" ]]; then
                    health="${GREEN}✓ running${NC}"
                elif curl -sf "http://localhost:$port/health" &>/dev/null; then
                    health="${GREEN}✓ healthy${NC}"
                else
                    health="${YELLOW}? unknown${NC}"
                fi
            else
                status="${YELLOW}loaded${NC}"
                health="-"
                pid="-"
            fi
        else
            status="${RED}not loaded${NC}"
            pid="-"
            health="-"
        fi
        
        printf "%-25s %-20b %-8s %b\n" "$name" "$status" "$pid" "$health"
    done
    
    # Watcher status
    local watcher_label="$PLIST_PREFIX.watcher"
    local watcher_status watcher_pid
    if launchctl list "$watcher_label" &>/dev/null; then
        watcher_pid=$(launchctl list | grep "$watcher_label" | awk '{print $1}')
        if [[ -n "$watcher_pid" && "$watcher_pid" != "-" ]]; then
            watcher_status="${GREEN}running${NC}"
        else
            watcher_status="${YELLOW}loaded${NC}"
            watcher_pid="-"
        fi
    else
        watcher_status="${RED}not loaded${NC}"
        watcher_pid="-"
    fi
    printf "%-25s %-20b %-8s %s\n" "watcher" "$watcher_status" "$watcher_pid" "-"
    
    echo ""
    echo "═══ Log Files ═══"
    echo "  $LOG_DIR/"
    find "$LOG_DIR" -maxdepth 1 -name "*.log" -exec stat -f "    %N (%z bytes)" {} \; 2>/dev/null || echo "    (no logs yet)"
}

# Tail logs
tail_logs() {
    log_info "Tailing all CTO service logs (Ctrl+C to stop)..."
    echo ""
    
    # Use tail with multiple files
    tail -f "$LOG_DIR"/*.log "$LOG_DIR"/*.err 2>/dev/null || {
        log_warn "No log files found yet. Services may not have started."
    }
}

# Restart all services
restart_services() {
    log_info "Restarting all CTO services..."
    
    local gui_domain
    gui_domain="gui/$(id -u)"
    
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        local label="$PLIST_PREFIX.$name"
        
        log_info "Restarting $name..."
        launchctl kickstart -k "$gui_domain/$label" 2>/dev/null || {
            log_warn "Could not restart $name (may not be loaded)"
        }
    done
    
    # Don't restart watcher unless needed
    log_info "✅ Services restarted!"
}

# Start services (if not running)
start_services() {
    log_info "Starting CTO services..."
    
    local gui_domain
    gui_domain="gui/$(id -u)"
    
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        local label="$PLIST_PREFIX.$name"
        
        log_info "Starting $name..."
        launchctl kickstart "$gui_domain/$label" 2>/dev/null || {
            log_warn "Could not start $name (may need to install first)"
        }
    done
    
    # Start watcher
    launchctl kickstart "$gui_domain/$PLIST_PREFIX.watcher" 2>/dev/null || true
    
    log_info "✅ Services started!"
}

# Stop services
stop_services() {
    log_info "Stopping CTO services..."
    
    local gui_domain
    gui_domain="gui/$(id -u)"
    
    for service_def in "${SERVICES[@]}"; do
        IFS=':' read -r name binary port <<< "$service_def"
        local label="$PLIST_PREFIX.$name"
        
        log_info "Stopping $name..."
        launchctl kill SIGTERM "$gui_domain/$label" 2>/dev/null || true
    done
    
    # Stop watcher
    launchctl kill SIGTERM "$gui_domain/$PLIST_PREFIX.watcher" 2>/dev/null || true
    
    log_info "✅ Services stopped!"
}

# Main
case "${1:-}" in
    install)
        install_services
        ;;
    uninstall)
        uninstall_services
        ;;
    status)
        show_status
        ;;
    logs)
        tail_logs
        ;;
    restart)
        restart_services
        ;;
    start)
        start_services
        ;;
    stop)
        stop_services
        ;;
    *)
        echo "Usage: $0 {install|uninstall|status|logs|restart|start|stop}"
        echo ""
        echo "Commands:"
        echo "  install    Generate plist files and load services"
        echo "  uninstall  Unload services and remove plist files"
        echo "  status     Show service status and health"
        echo "  logs       Tail all service logs"
        echo "  restart    Restart all services"
        echo "  start      Start services (if stopped)"
        echo "  stop       Stop services (without unloading)"
        exit 1
        ;;
esac
