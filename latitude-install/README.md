# Latitude Installer Ralph Loop

A dual-agent system for running and **progressively hardening** the CTO Platform bare metal installer on Latitude.sh infrastructure.

## Overview

This system uses two AI agents with distinct roles:

| Agent | CLI | Role |
|-------|-----|------|
| **Installer** | Claude | Runs the installer, handles issues during THIS run |
| **Hardening** | Droid | Watches Claude and implements CODE FIXES for NEXT run |

**The Key Insight**: Every time Claude has to solve a problem manually, Droid asks: "What code change would prevent this next time?" Then Droid implements that fix.

### Progressive Hardening Flow

```
Run N:   Claude encounters issue → figures out workaround → completes install
         ↓
         Droid observes: "Claude retried X 5 times"
         ↓
         Droid implements: Better retry logic in installer code
         ↓
Run N+1: Issue is handled automatically → Claude has less work
```

**Goal**: Each run makes the installer MORE reliable and LESS dependent on model intelligence.

Both agents have access to:
- **Latitude MCP** - Latitude.sh API for server management
- **Talos MCP** - Talos Linux operations (config apply, health checks)

## Quick Start

```bash
# Terminal 1: Start the installer agent (unattended mode)
./run-installer.sh

# Terminal 2: Start the monitor agent (waits for installer to be running)
./run-monitor.sh
```

### Unattended Mode (Default)

Both agents run in fully unattended mode by default:

- **Claude** uses `--dangerously-skip-permissions` to auto-approve all operations
- **Droid** uses `droid exec --skip-permissions-unsafe --auto high` for non-interactive execution

### Interactive Mode

For debugging or manual oversight:

```bash
# Interactive mode (prompts for approval)
./run-installer.sh --interactive
```

### Coordination

The monitor script automatically waits for the installer to start before beginning checks:

```bash
# Skip the wait (useful for debugging)
./run-monitor.sh --no-wait
```

## Files

| File | Description |
|------|-------------|
| `installer-prompt.md` | Instructions for Claude installer agent |
| `monitor-prompt.md` | Instructions for Droid monitor agent |
| `ralph-coordination.json` | Shared state between agents |
| `prd.json` | Test objectives and acceptance criteria (19 stories) |
| `progress.txt` | Human-readable progress log |
| `run-installer.sh` | Launch script for Claude |
| `run-monitor.sh` | Launch script for Droid |
| `cleanup.sh` | Reset state and optionally delete servers |

## Installation Steps

The installer runs through 23 steps in sequence:

### Infrastructure (Latitude API)
1. ValidatingPrerequisites - Check tools (talosctl, kubectl, helm)
2. CreatingServers - Provision bare metal via Latitude API
3. CreatingVLAN - Create private network for node communication
4. WaitingServersReady - Poll until servers are "on"
5. BootingTalos - Trigger iPXE reinstall with Talos image
6. WaitingTalosMaintenance - Wait for Talos maintenance mode

### Talos Bootstrap
7. GeneratingConfigs - Generate Talos secrets and machine configs
8. ApplyingCPConfig - Apply config to control plane
9. WaitingCPInstall - Wait for Talos installation
10. Bootstrapping - Bootstrap etcd and Kubernetes
11. DeployingCilium - Deploy CNI (required for node Ready)
12. WaitingKubernetes - Wait for K8s API ready
13. ApplyingWorkerConfig - Apply configs to workers
14. WaitingWorkerJoin - Wait for workers to join

### Platform Stack (GitOps)
15. DeployingBootstrapResources - Namespaces, RBAC
16. DeployingLocalPathProvisioner - Initial storage
17. DeployingArgoCD - GitOps controller
18. WaitingArgoCDReady - ArgoCD healthy
19. ApplyingAppOfApps - Deploy app-of-apps manifest
20. WaitingGitOpsSync - All apps synced

### Post-GitOps
21. ConfiguringStorage - Mayastor DiskPools
22. BootstrappingOpenBao - Secrets management
23. ConfiguringKubeconfig - Merge kubeconfig for kubectl/Lens

## MCP Servers

### Latitude MCP

Provides access to the Latitude.sh API:
- List/create/delete servers
- List/create VLANs
- Check server status
- List regions and plans

```bash
# Verify it's connected
claude mcp list | grep latitude
```

### Talos MCP

Provides Talos Linux operations:
- Apply machine configs
- Bootstrap cluster
- Health checks
- Service management

```bash
# Verify it's connected
claude mcp list | grep talos
```

## Cleanup

```bash
# Clean local state only (installer state, coordination file, progress log)
./cleanup.sh --local

# Full cleanup (guides through server deletion + local state)
./cleanup.sh --full
```

Local state includes:
- `/tmp/latitude-test/` - Installer state directory
- `ralph-coordination.json` - Reset to defaults
- `progress.txt` - Cleared
- Kubeconfig context `latitude-test` - Removed

## Coordination System

The agents coordinate via `ralph-coordination.json`:

```json
{
  "installer": {
    "status": "running|waiting|failed|complete",
    "currentStep": "CreatingServers",
    "lastUpdate": "2026-01-20T12:00:00Z"
  },
  "hardening": {
    "status": "running|idle",
    "lastCheck": "2026-01-20T12:00:00Z",
    "fixesImplemented": 0
  },
  "hardeningActions": [
    {
      "timestamp": "2026-01-20T12:00:00Z",
      "observation": "Claude retried Talos API connection 5 times manually",
      "rootCause": "Installer retry logic doesn't handle connection refused",
      "fix": "Added exponential backoff to orchestrator.rs",
      "files": ["crates/installer/src/orchestrator.rs"]
    }
  ],
  "circuitBreaker": {
    "state": "closed|open",
    "failureCount": 0,
    "threshold": 3
  }
}
```

### Hardening Actions

When Droid implements a code fix, it logs:
- **observation**: What Claude had to do manually
- **rootCause**: Why the code didn't handle it
- **fix**: What Droid changed
- **files**: Which files were modified

This creates a trail of improvements for each installation run.

## Default Cluster Configuration

| Setting | Value |
|---------|-------|
| Cluster name | `latitude-test` |
| Region | `DAL` (Dallas) |
| Control plane plan | `c2-small-x86` |
| Worker plan | `c2-small-x86` |
| Node count | 2 (1 CP + 1 worker) |
| Talos version | `v1.9.0` |

To change these, edit `ralph-coordination.json` before running, or pass different flags to the installer binary.

## Troubleshooting

### MCP Server Not Connected

```bash
# Re-add Latitude MCP to Claude
claude mcp add latitude -- npx -y latitudesh start --bearer <API_KEY>

# Re-add Talos MCP to Claude
claude mcp add talos-mcp /Users/jonathonfritz/.local/bin/talos-mcp-server
```

### Installer Stuck

1. Check `ralph-coordination.json` for current step and last update
2. Check `/tmp/latitude-test/install-state.json` for detailed state
3. Run monitor agent to get diagnostics
4. Use `./cleanup.sh --local` to reset and retry

### Server Stuck in "off" State

Latitude servers can occasionally get stuck. Options:
1. Wait longer (up to 15 minutes)
2. Delete the server via Latitude MCP and let installer recreate
3. Check Latitude dashboard for issues

### GitOps Sync Timeout

Default timeout is 30 minutes. Some operators take longer:
- GPU Operator
- Mayastor
- Large CRD installations

Check ArgoCD for which apps are still syncing:
```bash
kubectl --kubeconfig /tmp/latitude-test/kubeconfig get applications -n argocd
```

## Related Documentation

- [Installer source](../crates/installer/) - Rust installer binary
- [Metal crate](../crates/metal/) - Latitude API client and Talos operations
- [GitOps applications](../infra/gitops/applications/) - What gets deployed via ArgoCD
- [Lifecycle test](../lifecycle-test/) - Similar Ralph loop for CTO platform testing
