#!/bin/bash
# Manage the SWARM puzzle watcher as a per-user launchd agent.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RUNNER_PATH="$PROJECT_DIR/scripts/swarm_puzzle_runner.py"

LABEL="ai.5dlabs.swarm-puzzle-watcher"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/$LABEL.plist"
LOG_DIR="/tmp/swarm-puzzle-launchd"
STDOUT_LOG="$LOG_DIR/swarm-puzzle.log"
STDERR_LOG="$LOG_DIR/swarm-puzzle.err"
GUI_DOMAIN="gui/$(id -u)"

DEFAULT_ACCOUNT="jonathon@5dlabs.ai"
DEFAULT_INTERVAL="300"
DEFAULT_RUNTIME_ROOT="$HOME/.swarm-puzzle-agent"
DEFAULT_WORKSPACE_ROOT="$PROJECT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

usage() {
    cat <<EOF
Usage:
  $0 install [--account EMAIL] [--interval SECONDS] [--runtime-root DIR] [--workspace-root DIR] [--model MODEL] [--dry-run]
  $0 uninstall
  $0 status
  $0 logs
  $0 restart
  $0 test-bundle [--bundle-dir DIR] [--runtime-root DIR] [--workspace-root DIR] [--model MODEL]

Commands:
  install      Generate and load the launchd plist.
  uninstall    Unload and remove the launchd plist.
  status       Show launchd status and log locations.
  logs         Tail watcher logs.
  restart      Kickstart the watcher.
  test-bundle  Replay a saved puzzle bundle without polling Gmail or submitting.
EOF
}

require_runner() {
    if [[ ! -f "$RUNNER_PATH" ]]; then
        log_error "Runner not found at $RUNNER_PATH"
        exit 1
    fi
}

find_python3() {
    command -v python3 || true
}

build_path_env() {
    local -a pieces=(
        "/opt/homebrew/bin"
        "/usr/local/bin"
        "/usr/bin"
        "/bin"
        "/usr/sbin"
        "/sbin"
        "$HOME/.local/bin"
        "$HOME/.bun/bin"
        "$HOME/Library/pnpm"
        "$HOME/.cargo/bin"
        "/Applications/Codex.app/Contents/Resources"
    )
    local tool
    for tool in gog swarm codex python3; do
        if command -v "$tool" >/dev/null 2>&1; then
            pieces+=("$(dirname "$(command -v "$tool")")")
        fi
    done

    local result=""
    local piece
    local seen=":"
    for piece in "${pieces[@]}"; do
        [[ -n "$piece" ]] || continue
        if [[ "$seen" != *":$piece:"* ]]; then
            if [[ -n "$result" ]]; then
                result="$result:$piece"
            else
                result="$piece"
            fi
            seen="${seen}${piece}:"
        fi
    done
    printf '%s\n' "$result"
}

xml_escape() {
    local value="$1"
    value="${value//&/&amp;}"
    value="${value//</&lt;}"
    value="${value//>/&gt;}"
    printf '%s' "$value"
}

emit_arg() {
    printf '        <string>%s</string>\n' "$(xml_escape "$1")"
}

generate_plist() {
    local account="$1"
    local interval="$2"
    local runtime_root="$3"
    local workspace_root="$4"
    local model="$5"
    local dry_run="$6"
    local python3_path="$7"
    local path_env="$8"

    cat <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LABEL</string>

    <key>ProgramArguments</key>
    <array>
$(emit_arg "$python3_path")$(emit_arg "$RUNNER_PATH")$(emit_arg "--account")$(emit_arg "$account")$(emit_arg "--watch")$(emit_arg "--interval")$(emit_arg "$interval")$(emit_arg "--runtime-root")$(emit_arg "$runtime_root")$(emit_arg "--workspace-root")$(emit_arg "$workspace_root")$(if [[ -n "$model" ]]; then emit_arg "--model"; emit_arg "$model"; fi)$(if [[ "$dry_run" == "true" ]]; then emit_arg "--dry-run"; fi)    </array>

    <key>WorkingDirectory</key>
    <string>$PROJECT_DIR</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>HOME</key>
        <string>$HOME</string>
        <key>LANG</key>
        <string>en_US.UTF-8</string>
        <key>LC_ALL</key>
        <string>en_US.UTF-8</string>
        <key>PATH</key>
        <string>$path_env</string>
        <key>PYTHONUNBUFFERED</key>
        <string>1</string>
    </dict>

    <key>KeepAlive</key>
    <true/>

    <key>RunAtLoad</key>
    <true/>

    <key>StandardOutPath</key>
    <string>$STDOUT_LOG</string>

    <key>StandardErrorPath</key>
    <string>$STDERR_LOG</string>

    <key>ThrottleInterval</key>
    <integer>30</integer>
</dict>
</plist>
EOF
}

status_service() {
    local loaded="no"
    if launchctl list | grep -q "$LABEL"; then
        loaded="yes"
    fi

    echo "Label: $LABEL"
    echo "Loaded: $loaded"
    echo "Plist: $PLIST_PATH"
    echo "Stdout: $STDOUT_LOG"
    echo "Stderr: $STDERR_LOG"

    if [[ "$loaded" == "yes" ]]; then
        echo
        launchctl print "$GUI_DOMAIN/$LABEL" 2>/dev/null | sed -n '1,80p'
    fi
}

install_service() {
    local account="$DEFAULT_ACCOUNT"
    local interval="$DEFAULT_INTERVAL"
    local runtime_root="$DEFAULT_RUNTIME_ROOT"
    local workspace_root="$DEFAULT_WORKSPACE_ROOT"
    local model=""
    local dry_run="false"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --account)
                account="$2"
                shift 2
                ;;
            --interval)
                interval="$2"
                shift 2
                ;;
            --runtime-root)
                runtime_root="$2"
                shift 2
                ;;
            --workspace-root)
                workspace_root="$2"
                shift 2
                ;;
            --model)
                model="$2"
                shift 2
                ;;
            --dry-run)
                dry_run="true"
                shift
                ;;
            *)
                log_error "Unknown install option: $1"
                usage
                exit 1
                ;;
        esac
    done

    require_runner

    local python3_path
    python3_path="$(find_python3)"
    if [[ -z "$python3_path" ]]; then
        log_error "python3 not found in PATH"
        exit 1
    fi

    local path_env
    path_env="$(build_path_env)"

    mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR" "$runtime_root"
    generate_plist "$account" "$interval" "$runtime_root" "$workspace_root" "$model" "$dry_run" "$python3_path" "$path_env" > "$PLIST_PATH"

    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    launchctl load "$PLIST_PATH"

    log_info "Installed $LABEL"
    if [[ "$dry_run" == "true" ]]; then
        log_warn "The watcher is running in dry-run mode and will not submit answers."
    fi
    status_service
}

uninstall_service() {
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    rm -f "$PLIST_PATH"
    log_info "Removed $LABEL"
}

restart_service() {
    if [[ ! -f "$PLIST_PATH" ]]; then
        log_error "Plist not found at $PLIST_PATH"
        exit 1
    fi
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    launchctl load "$PLIST_PATH"
    log_info "Restarted $LABEL"
}

logs_service() {
    mkdir -p "$LOG_DIR"
    touch "$STDOUT_LOG" "$STDERR_LOG"
    tail -n 100 -F "$STDOUT_LOG" "$STDERR_LOG"
}

find_latest_bundle() {
    local runtime_root="$1"
    local latest
    latest="$(find "$runtime_root/bundles" -mindepth 1 -maxdepth 1 -type d -print 2>/dev/null | sort | tail -n 1 || true)"
    if [[ -z "$latest" ]]; then
        log_error "No saved bundles found under $runtime_root/bundles"
        exit 1
    fi
    printf '%s\n' "$latest"
}

test_bundle() {
    local bundle_dir=""
    local runtime_root="$DEFAULT_RUNTIME_ROOT"
    local workspace_root="$DEFAULT_WORKSPACE_ROOT"
    local model=""
    local -a cmd

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --bundle-dir)
                bundle_dir="$2"
                shift 2
                ;;
            --runtime-root)
                runtime_root="$2"
                shift 2
                ;;
            --workspace-root)
                workspace_root="$2"
                shift 2
                ;;
            --model)
                model="$2"
                shift 2
                ;;
            *)
                log_error "Unknown test-bundle option: $1"
                usage
                exit 1
                ;;
        esac
    done

    require_runner

    if [[ -z "$bundle_dir" ]]; then
        bundle_dir="$(find_latest_bundle "$runtime_root")"
    fi

    cmd=(
        python3
        "$RUNNER_PATH"
        --bundle-dir "$bundle_dir"
        --runtime-root "$runtime_root"
        --workspace-root "$workspace_root"
    )
    if [[ -n "$model" ]]; then
        cmd+=(--model "$model")
    fi
    "${cmd[@]}"
}

main() {
    if [[ $# -lt 1 ]]; then
        usage
        exit 1
    fi

    local command="$1"
    shift

    case "$command" in
        install)
            install_service "$@"
            ;;
        uninstall)
            uninstall_service
            ;;
        status)
            status_service
            ;;
        logs)
            logs_service
            ;;
        restart)
            restart_service
            ;;
        test-bundle)
            test_bundle "$@"
            ;;
        *)
            log_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

main "$@"
