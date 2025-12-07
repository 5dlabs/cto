#!/usr/bin/env bash
# Kilo VPN Connection Script
# Usage: kilo-vpn.sh [connect|disconnect|status]
#
# This script manages WireGuard VPN connections to the cluster.
# Use this instead of mixing the WireGuard GUI app with wg-quick.

set -euo pipefail

KILO_CONF="${KILO_CONF:-$HOME/.wireguard/kilo.conf}"
SCRIPT_NAME="$(basename "$0")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_config() {
    if [[ ! -f "$KILO_CONF" ]]; then
        log_error "Kilo config not found at: $KILO_CONF"
        log_info "See docs/vpn/kilo-client-setup.md for setup instructions"
        exit 1
    fi
}

check_wireguard_app() {
    # Check if WireGuard.app is managing any tunnels
    if pgrep -f "WireGuardNetworkExtension" > /dev/null 2>&1; then
        local wg_output
        wg_output=$(sudo wg show 2>/dev/null || true)
        if [[ -n "$wg_output" ]]; then
            log_warn "WireGuard.app may have an active tunnel"
            log_warn "Disable the tunnel in WireGuard.app before using this script"
            log_warn "Or quit WireGuard.app entirely to avoid conflicts"
            return 1
        fi
    fi
    return 0
}

get_interface() {
    # Get the WireGuard interface name from wg show
    sudo wg show 2>/dev/null | head -1 | awk '{print $2}' || true
}

cleanup_stale_routes() {
    # Clean up stale host routes that may have been left by WireGuard.app
    # These override network routes and break connectivity
    log_info "Cleaning up stale routes..."
    
    # Common stale routes from WireGuard.app
    local stale_routes=(
        "10.5.0.10"    # Peer IP
        "10.96.0.10"   # CoreDNS
    )
    
    for route in "${stale_routes[@]}"; do
        if netstat -rn 2>/dev/null | grep -q "^${route}.*utun"; then
            sudo route delete -host "$route" 2>/dev/null || true
        fi
    done
    
    # Also clean any routes pointing to non-existent utun interfaces
    # (interfaces that were created by WireGuard.app but no longer exist)
    local current_iface
    current_iface=$(get_interface)
    
    for i in {0..10}; do
        local old_iface="utun${i}"
        if [[ "$old_iface" != "$current_iface" ]] && [[ -z "$current_iface" || "$old_iface" != "$current_iface" ]]; then
            # Check if any cluster routes point to this old interface
            if netstat -rn 2>/dev/null | grep -E "^10\.(96|244|4|5)" | grep -q "$old_iface"; then
                log_warn "Found stale routes on $old_iface, cleaning..."
                sudo route delete -net 10.96.0.0/12 2>/dev/null || true
                sudo route delete -net 10.244.0.0/16 2>/dev/null || true
                sudo route delete -net 10.4.0.0/24 2>/dev/null || true
                sudo route delete -net 10.5.0.0/24 2>/dev/null || true
                break
            fi
        fi
    done
}

connect() {
    check_config
    
    # Check for existing connection
    local existing_iface
    existing_iface=$(get_interface)
    if [[ -n "$existing_iface" ]]; then
        log_info "Already connected on interface: $existing_iface"
        status
        return 0
    fi
    
    # Warn about WireGuard.app conflicts
    if ! check_wireguard_app; then
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Clean up any stale routes before connecting
    cleanup_stale_routes
    
    log_info "Connecting to Kilo VPN..."
    sudo wg-quick up "$KILO_CONF"
    
    # Verify connection
    sleep 1
    if sudo wg show | grep -q "latest handshake"; then
        log_info "Connected successfully!"
        status
    else
        log_warn "Tunnel is up but no handshake yet. Waiting..."
        sleep 3
        if sudo wg show | grep -q "latest handshake"; then
            log_info "Handshake established!"
            status
        else
            log_warn "Still no handshake. Check your peer configuration."
        fi
    fi
}

disconnect() {
    local existing_iface
    existing_iface=$(get_interface)
    
    if [[ -z "$existing_iface" ]]; then
        log_info "Not connected"
        return 0
    fi
    
    log_info "Disconnecting from Kilo VPN..."
    sudo wg-quick down "$KILO_CONF" 2>/dev/null || true
    log_info "Disconnected"
}

status() {
    local iface
    iface=$(get_interface)
    
    if [[ -z "$iface" ]]; then
        log_info "Status: Disconnected"
        return 0
    fi
    
    echo ""
    log_info "Status: Connected on $iface"
    echo ""
    
    # Show WireGuard status
    sudo wg show
    echo ""
    
    # Test cluster connectivity
    log_info "Testing cluster connectivity..."
    
    # Get ArgoCD ClusterIP
    local argocd_ip
    argocd_ip=$(kubectl get svc -n argocd argocd-server -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "")
    
    if [[ -n "$argocd_ip" ]]; then
        if curl -s --connect-timeout 2 "http://${argocd_ip}" -o /dev/null; then
            log_info "  ArgoCD ($argocd_ip): ✓ Reachable"
        else
            log_error "  ArgoCD ($argocd_ip): ✗ Unreachable"
        fi
    fi
    
    # Get Argo Workflows ClusterIP
    local workflows_ip
    workflows_ip=$(kubectl get svc -n automation argo-workflows-server -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "")
    
    if [[ -n "$workflows_ip" ]]; then
        if curl -s --connect-timeout 2 "http://${workflows_ip}:2746" -o /dev/null; then
            log_info "  Argo Workflows ($workflows_ip): ✓ Reachable"
        else
            log_error "  Argo Workflows ($workflows_ip): ✗ Unreachable"
        fi
    fi
    
    # Test DNS resolution
    local dns_ip
    dns_ip=$(kubectl get svc -n kube-system kube-dns -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "10.96.0.10")
    
    if dig +short +time=2 "@${dns_ip}" kubernetes.default.svc.cluster.local > /dev/null 2>&1; then
        log_info "  Cluster DNS ($dns_ip): ✓ Resolving"
    else
        log_error "  Cluster DNS ($dns_ip): ✗ Not resolving"
    fi
    
    echo ""
}

usage() {
    cat <<EOF
Usage: $SCRIPT_NAME [command]

Commands:
    connect      Connect to Kilo VPN
    disconnect   Disconnect from Kilo VPN
    status       Show connection status and test connectivity
    help         Show this help message

Environment Variables:
    KILO_CONF    Path to WireGuard config (default: ~/.wireguard/kilo.conf)

Examples:
    $SCRIPT_NAME connect
    $SCRIPT_NAME status
    KILO_CONF=/path/to/config.conf $SCRIPT_NAME connect

Notes:
    - Requires sudo for WireGuard operations
    - Do not use with WireGuard.app simultaneously (causes route conflicts)
    - See docs/vpn/kilo-client-setup.md for initial setup
EOF
}

main() {
    local cmd="${1:-status}"
    
    case "$cmd" in
        connect|up)
            connect
            ;;
        disconnect|down)
            disconnect
            ;;
        status|show)
            status
            ;;
        help|-h|--help)
            usage
            ;;
        *)
            log_error "Unknown command: $cmd"
            usage
            exit 1
            ;;
    esac
}

main "$@"

