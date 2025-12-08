# GitOps Cleanup Strategy

> **Status**: ✅ Implemented

## Current State Analysis

| Directory | Type | Purpose | Kustomize Features Used |
|-----------|------|---------|------------------------|
| `cloudflare-operator/` | Kustomize | Pull upstream + patch | ✅ Remote base + patches |
| `cloudflare-tunnel/` | ~~Raw YAML~~ | ~~Tunnel bindings~~ | ✅ Moved to CTO chart |
| `kilo/` | Kustomize | List resources | ❌ Just listing files |
| `test-databases/` | Kustomize | Set namespace | ✅ Namespace transform |
| `external-secrets/` | Raw YAML | ExternalSecrets | N/A |
| `openbao/` | Raw YAML | RBAC only | N/A |
| `external-dns/` | Empty | Just README | N/A |
| `rbac/` | Raw YAML | Platform RBAC | N/A |

## Recommended Changes

### 1. Keep Kustomize where it adds value
- ✅ `cloudflare-operator/` - Pulls from upstream GitHub and patches
- ✅ `test-databases/` - Uses namespace transformation

### 2. Remove unnecessary Kustomize
- ❌ `cloudflare-tunnel/` - Remove `kustomization.yaml`
- ❌ `kilo/` - Remove `kustomization.yaml`

### 3. Consolidate RBAC
- Move `rbac/` files into `manifests/rbac/`
- Update `rbac-configs` ArgoCD Application path

### 4. Clean up empty directories
- Delete `external-dns/` (just a README)

## Target Structure

```
infra/
├── charts/
│   └── cto/                    # Single CTO platform Helm chart
│
└── gitops/
    ├── applications/           # ArgoCD Application definitions
    │   ├── observability/
    │   ├── operators/
    │   ├── platform/
    │   ├── secrets/
    │   └── workloads/
    ├── manifests/
    │   ├── cloudflare-operator/ # Kustomize (upstream + patches)
    │   ├── cloudflare-tunnel/   # Raw YAML
    │   ├── external-secrets/    # Raw YAML
    │   ├── kilo/               # Raw YAML
    │   ├── openbao/            # Raw YAML
    │   ├── rbac/               # Consolidated platform RBAC
    │   └── test-databases/     # Kustomize (namespace transform)
    └── projects/               # ArgoCD Projects
```

## Rationale

- **Helm over Kustomize**: Standardize on Helm for templating, use raw YAML for simple configs
- **Kustomize only when needed**: Keep Kustomize for upstream pulls with patches or namespace transforms
- **ArgoCD directory mode**: ArgoCD can deploy raw YAML directories without Kustomize
- **Single CTO chart**: All CTO components in one distributable Helm chart


