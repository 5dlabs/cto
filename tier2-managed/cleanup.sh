#!/usr/bin/env bash
set -euo pipefail

# Tier 2 Managed Cleanup Script
# Cleans up local state and optionally infrastructure

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

CLEANUP_MODE=""
TENANT_ID=""

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Clean up Tier 2 Managed provisioning state.

Options:
    --local             Clean local state only (state files, logs)
    --full              Full cleanup (local + infrastructure deletion guidance)
    --tenant <id>       Tenant ID (reads from coordination file if not provided)
    -h, --help          Show this help message

Local cleanup includes:
    - /tmp/tier2-{tenant}/ directory
    - ralph-coordination.json (reset to defaults)
    - progress.txt (cleared)
    - .installer.pid and .monitor.pid files
    - Kubeconfig context removal

Full cleanup additionally:
    - Guides through server deletion via Provider MCP
    - Guides through VLAN deletion
    - Guides through WARP tunnel removal
    - Guides through ClusterMesh peering removal

Examples:
    # Clean local state only
    ./cleanup.sh --local

    # Full cleanup with infrastructure guidance
    ./cleanup.sh --full

    # Clean specific tenant
    ./cleanup.sh --local --tenant acme
EOF
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            CLEANUP_MODE="local"
            shift
            ;;
        --full)
            CLEANUP_MODE="full"
            shift
            ;;
        --tenant)
            TENANT_ID="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# Validate mode
if [[ -z "$CLEANUP_MODE" ]]; then
    echo -e "${RED}Error: Must specify --local or --full${NC}"
    usage
fi

# Get tenant ID from coordination file if not provided
if [[ -z "$TENANT_ID" ]]; then
    TENANT_ID=$(jq -r '.tenant.id' "$SCRIPT_DIR/ralph-coordination.json" 2>/dev/null || echo "")
fi

if [[ -z "$TENANT_ID" || "$TENANT_ID" == "example-tenant" ]]; then
    echo -e "${YELLOW}Warning: No tenant ID found. Using 'unknown' for local cleanup.${NC}"
    TENANT_ID="unknown"
fi

echo -e "${GREEN}Tier 2 Managed Cleanup${NC}"
echo "  Mode: $CLEANUP_MODE"
echo "  Tenant: $TENANT_ID"
echo ""

# Full cleanup guidance (infrastructure)
if [[ "$CLEANUP_MODE" == "full" ]]; then
    echo -e "${BLUE}=== Infrastructure Cleanup Guidance ===${NC}"
    echo ""
    
    # Read server info from coordination file
    CP_SERVERS=$(jq -r '.servers.controlPlanes[]?.id // empty' "$SCRIPT_DIR/ralph-coordination.json" 2>/dev/null || echo "")
    WORKER_SERVERS=$(jq -r '.servers.workers[]?.id // empty' "$SCRIPT_DIR/ralph-coordination.json" 2>/dev/null || echo "")
    VLAN_ID=$(jq -r '.vlan.id // empty' "$SCRIPT_DIR/ralph-coordination.json" 2>/dev/null || echo "")
    TUNNEL_ID=$(jq -r '.connectivity.warpConnector.tunnelId // empty' "$SCRIPT_DIR/ralph-coordination.json" 2>/dev/null || echo "")
    PROVIDER=$(jq -r '.tenant.provider // "latitude"' "$SCRIPT_DIR/ralph-coordination.json" 2>/dev/null || echo "latitude")
    
    echo -e "${YELLOW}1. Delete Servers via Provider MCP${NC}"
    if [[ -n "$CP_SERVERS" || -n "$WORKER_SERVERS" ]]; then
        echo "   Control Plane servers:"
        for srv in $CP_SERVERS; do
            echo "     - $srv"
        done
        echo "   Worker servers:"
        for srv in $WORKER_SERVERS; do
            echo "     - $srv"
        done
        echo ""
        echo "   Use Provider MCP to delete each server:"
        echo "   Example (Latitude): latitude_delete_server server_id=<id>"
    else
        echo "   No servers found in coordination state."
    fi
    echo ""
    
    echo -e "${YELLOW}2. Delete VLAN via Provider MCP${NC}"
    if [[ -n "$VLAN_ID" ]]; then
        echo "   VLAN ID: $VLAN_ID"
        echo "   Use Provider MCP: latitude_delete_vlan vlan_id=$VLAN_ID"
    else
        echo "   No VLAN found in coordination state."
    fi
    echo ""
    
    echo -e "${YELLOW}3. Remove WARP Tunnel from Cloudflare${NC}"
    if [[ -n "$TUNNEL_ID" ]]; then
        echo "   Tunnel ID: $TUNNEL_ID"
        echo "   Run: cloudflared tunnel delete $TUNNEL_ID"
        echo "   Or delete via Cloudflare Zero Trust dashboard."
    else
        echo "   No tunnel found in coordination state."
        echo "   Check Cloudflare dashboard for tunnels named: cto-$TENANT_ID-*"
    fi
    echo ""
    
    echo -e "${YELLOW}4. Remove ClusterMesh Peering${NC}"
    echo "   If ClusterMesh was configured, disconnect from control plane:"
    echo "   Run: cilium clustermesh disconnect --destination-context control-plane"
    echo ""
    
    echo -e "${YELLOW}5. Verify No Orphaned Resources${NC}"
    echo "   Check provider dashboard for any remaining:"
    echo "   - Servers with 'cto-$TENANT_ID' in hostname"
    echo "   - VLANs in the same region"
    echo ""
    
    read -p "Press Enter to continue with local cleanup, or Ctrl+C to abort..."
    echo ""
fi

# Local cleanup
echo -e "${BLUE}=== Local State Cleanup ===${NC}"
echo ""

# 1. Remove state directory
STATE_DIR="/tmp/tier2-${TENANT_ID}"
if [[ -d "$STATE_DIR" ]]; then
    echo -e "${YELLOW}Removing state directory: $STATE_DIR${NC}"
    rm -rf "$STATE_DIR"
    echo "  Done."
else
    echo "State directory not found: $STATE_DIR (skipping)"
fi

# 2. Remove kubeconfig context
CONTEXT_NAME="${TENANT_ID}-prod"
if kubectl config get-contexts "$CONTEXT_NAME" &>/dev/null; then
    echo -e "${YELLOW}Removing kubeconfig context: $CONTEXT_NAME${NC}"
    kubectl config delete-context "$CONTEXT_NAME" 2>/dev/null || true
    kubectl config delete-cluster "$CONTEXT_NAME" 2>/dev/null || true
    kubectl config delete-user "$CONTEXT_NAME" 2>/dev/null || true
    echo "  Done."
else
    echo "Kubeconfig context not found: $CONTEXT_NAME (skipping)"
fi

# 3. Remove PID files
echo -e "${YELLOW}Removing PID files${NC}"
rm -f "$SCRIPT_DIR/.installer.pid"
rm -f "$SCRIPT_DIR/.monitor.pid"
echo "  Done."

# 4. Clear progress log
echo -e "${YELLOW}Clearing progress.txt${NC}"
cat > "$SCRIPT_DIR/progress.txt" <<EOF
# Tier 2 Managed Dedicated Progress Log

This file is updated by the Installer and Monitor agents during provisioning.

---

## Session Not Started

Run \`./run-installer.sh\` to begin provisioning.
Run \`./run-monitor.sh\` in a separate terminal to start monitoring.

---
EOF
echo "  Done."

# 5. Reset coordination file
echo -e "${YELLOW}Resetting ralph-coordination.json${NC}"
cat > "$SCRIPT_DIR/ralph-coordination.json" <<EOF
{
  "installer": {
    "status": "pending",
    "currentStep": null,
    "stepNumber": 0,
    "totalSteps": 31,
    "lastUpdate": null,
    "lastError": null,
    "attemptCount": 0,
    "pid": null
  },
  "monitor": {
    "status": "idle",
    "lastCheck": null,
    "checkCount": 0,
    "pid": null
  },
  "tenant": {
    "id": "example-tenant",
    "provider": "latitude",
    "region": "DAL",
    "size": "medium"
  },
  "credentials": {
    "secretPath": "tenants/example-tenant/provider-creds",  # pragma: allowlist secret
    "validated": false,
    "validatedAt": null
  },
  "cluster": {
    "name": null,
    "kubeconfig": null,
    "talosconfig": null,
    "endpoint": null,
    "clusterMeshId": null
  },
  "servers": {
    "controlPlanes": [],
    "workers": []
  },
  "vlan": {
    "id": null,
    "vid": null,
    "cidr": null
  },
  "connectivity": {
    "warpConnector": {
      "status": "pending",
      "tunnelId": null,
      "tunnelName": null,
      "registeredAt": null
    },
    "clusterMesh": {
      "status": "pending",
      "peeredAt": null,
      "controlPlaneEndpoint": null
    },
    "l3Verified": false,
    "globalServicesVerified": false
  },
  "platform": {
    "argocd": {
      "deployed": false,
      "healthy": false,
      "adminPasswordSet": false
    },
    "agentController": {
      "deployed": false,
      "healthy": false
    },
    "fleetRegistered": false,
    "fleetRegistrationId": null
  },
  "verification": {
    "codeRunDispatched": false,
    "codeRunCompleted": false,
    "statusSyncVerified": false,
    "dashboardConfigured": false
  },
  "issueQueue": [],
  "circuitBreaker": {
    "state": "closed",
    "failureCount": 0,
    "sameStepFailures": 0,
    "lastFailedStep": null,
    "lastError": null,
    "openedAt": null,
    "threshold": 3
  },
  "session": {
    "id": null,
    "startedAt": null,
    "lastActivity": null,
    "installerCli": "claude",
    "monitorCli": "claude"
  },
  "stats": {
    "totalIssues": 0,
    "resolvedIssues": 0,
    "failedAttempts": 0,
    "successfulSteps": 0,
    "totalDuration": null
  }
}
EOF
echo "  Done."

echo ""
echo -e "${GREEN}Cleanup complete!${NC}"
echo ""
echo "To start a fresh provisioning run:"
echo "  1. ./run-installer.sh --tenant <tenant-id>"
echo "  2. ./run-monitor.sh (in separate terminal)"
