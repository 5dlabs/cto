# GitOps Deployment Ralph Loop

A dual-agent system for deploying and syncing ArgoCD applications on a Kubernetes cluster.

## Overview

This Ralph loop handles getting all GitOps applications deployed, synced, and healthy after the Kubernetes cluster is up.

### Agents

| Agent | CLI | Role |
|-------|-----|------|
| **Claude** | `claude` | Deployer - syncs apps, fixes sync failures |
| **Droid** | `droid` | Hardener - improves GitOps configs based on failures |

### Prerequisites

- Kubernetes cluster running (from installer Ralph loop)
- ArgoCD deployed and accessible
- `kubeconfig` available at `/tmp/latitude-test/kubeconfig`

## Quick Start

```bash
# Start the tmux session
./tmux-session.sh

# In top pane (Claude): run the deployer
./run-deployer.sh

# In middle pane (Droid): run the hardener (after deployer starts)
./run-hardener.sh
```

## Workflow

```
┌─────────────────────────────────────────────────────┐
│                   GitOps Deploy Loop                │
├─────────────────────────────────────────────────────┤
│  1. Apply platform project + app-of-apps           │
│  2. Wait for applications to be created            │
│  3. Sync apps in dependency order:                 │
│     - cert-manager (TLS)                           │
│     - external-secrets (secrets from OpenBao)      │
│     - operators (database, storage, etc.)          │
│     - platform services                            │
│  4. Verify all apps Synced + Healthy               │
│  5. Report any failures for Droid to harden        │
└─────────────────────────────────────────────────────┘
```

## Success Criteria

- [ ] All ArgoCD applications show `Synced`
- [ ] All ArgoCD applications show `Healthy`
- [ ] No pods in CrashLoopBackOff or Error state
- [ ] Core services responding (cert-manager, external-secrets, ArgoCD)

## Files

| File | Purpose |
|------|---------|
| `deployer-prompt.md` | Claude's instructions |
| `hardener-prompt.md` | Droid's instructions |
| `ralph-coordination.json` | Shared state between agents |
| `progress.txt` | Human-readable log |
| `lessons-learned.md` | Documented fixes |

## Known Dependencies

Some apps depend on others being healthy first:

```
cert-manager → external-secrets → platform secrets
operators → database instances
cilium → network policies
```
