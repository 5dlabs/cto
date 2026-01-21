# CTO ArgoCD GitOps Repository

This repository contains ArgoCD Application manifests for your CTO Managed Dedicated cluster.

## Overview

ArgoCD on your cluster syncs configuration from this repository. When you make changes here, they are automatically applied to your cluster.

## Structure

```
.
├── README.md                    # This file
├── values.yaml                  # Tenant-specific configuration
├── applications/
│   ├── app-of-apps.yaml         # Root Application (syncs all others)
│   ├── agent-controller.yaml    # CTO Agent Controller
│   ├── mcp-tools.yaml           # MCP tool sidecars
│   ├── cilium.yaml              # CNI with ClusterMesh
│   └── local-path-provisioner.yaml  # Storage provisioner
└── base/
    ├── namespaces.yaml          # Cluster namespaces
    └── rbac.yaml                # RBAC configuration
```

## How It Works

1. **App-of-Apps Pattern**: The root `app-of-apps.yaml` Application syncs all other Applications.
2. **Auto-Sync**: Runtime components auto-sync when manifests change.
3. **Manual Sync**: Infrastructure changes require manual sync for safety.
4. **Values Override**: Edit `values.yaml` to customize your deployment.

## Configuration

Edit `values.yaml` to customize your cluster:

```yaml
tenant:
  id: your-tenant-id
  provider: latitude
  region: DAL
  clusterSize: medium

cluster:
  name: your-cluster-name

# Enable/disable applications
applications:
  - name: agent-controller
    enabled: true
  - name: mcp-tools
    enabled: true
```

## Updating

When 5D Labs releases updates:
1. A PR will be created against this repo with updated manifests
2. Review and merge the PR
3. ArgoCD automatically syncs the changes to your cluster

## Support

- Documentation: https://docs.5dlabs.ai
- Support: support@5dlabs.ai
- Status: https://status.5dlabs.ai

---

*This repository is managed by CTO. Do not delete or rename without contacting support.*
