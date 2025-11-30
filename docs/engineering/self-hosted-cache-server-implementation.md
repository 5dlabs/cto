# Self-Hosted GitHub Actions Cache Server Implementation

## Overview

This document outlines the implementation plan for deploying the [falcondev-oss/github-actions-cache-server](https://github.com/falcondev-oss/github-actions-cache-server) as a self-hosted replacement for GitHub's hosted cache service.

## Current Caching Architecture

### GitHub Hosted Runners

Workflows running on `ubuntu-latest` or `ubuntu-22.04` use:

| Action | Purpose | Files |
|--------|---------|-------|
| `actions/cache@v4` | kubectl, Trivy DB, cargo-audit, cargo-llvm-cov | `controller-ci.yaml`, `tools-ci.yaml` |
| `Swatinem/rust-cache@v2` | Rust target directories, cargo registry | All Rust CI workflows |

### Self-Hosted Runners (Kubernetes)

Self-hosted runners in namespace `arc-runners` use PVC-based caching:

```yaml
# Current PVCs
- runner-cache-pvc: 200Gi (general cache)
- rust-cache-pvc: 100Gi (Rust-specific)

# Mount Points
- /cache/cargo     → CARGO_HOME
- /cache/rustup    → RUSTUP_HOME
- /cache/sccache   → sccache build cache
- /cache/target    → CARGO_TARGET_DIR
```

**Key limitation:** Self-hosted runners cannot use `actions/cache` without a cache server.

## Why Self-Hosted Cache Server?

### Benefits

1. **Unified caching strategy** - Use `actions/cache` consistently across GitHub and self-hosted runners
2. **Better cache isolation** - Per-workflow/job cache scopes instead of shared PVC
3. **GitHub Actions compatibility** - Works transparently with `actions/cache`, `Swatinem/rust-cache`
4. **Storage flexibility** - S3, MinIO, local filesystem, or other storage backends
5. **Cost control** - No GitHub cache storage limits (10GB free tier)
6. **Network locality** - Cache server runs in-cluster for fast access

### Trade-offs

| Current (PVC) | Self-Hosted Cache Server |
|---------------|--------------------------|
| Simple, no extra service | Additional deployment to manage |
| Single shared volume | HTTP API overhead |
| Manual cleanup needed | Automatic cache eviction |
| No restore-keys matching | Full GitHub cache semantics |

## Implementation Plan

### Phase 1: Deploy Cache Server

#### 1.1 Create Helm Chart

Create `infra/charts/gha-cache-server/`:

```yaml
# values.yaml
replicaCount: 1

image:
  repository: ghcr.io/falcondev-oss/github-actions-cache-server
  tag: "v8.1.4"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 3000

storage:
  # Options: local, s3, minio
  type: local
  local:
    path: /app/.data
    persistence:
      enabled: true
      size: 100Gi
      storageClass: local-path

env:
  API_BASE_URL: "http://gha-cache-server.arc-systems.svc.cluster.local:3000"
  # For S3/MinIO:
  # STORAGE_DRIVER: s3
  # S3_BUCKET: gha-cache
  # S3_REGION: us-east-1
  # S3_ENDPOINT: http://minio.storage.svc.cluster.local:9000

resources:
  limits:
    cpu: "1"
    memory: "1Gi"
  requests:
    cpu: "200m"
    memory: "256Mi"
```

#### 1.2 ArgoCD Application

```yaml
# infra/gitops/applications/gha-cache-server.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: gha-cache-server
  namespace: argocd
spec:
  project: platform
  source:
    repoURL: https://github.com/5dlabs/cto
    targetRevision: main
    path: infra/charts/gha-cache-server
  destination:
    server: https://kubernetes.default.svc
    namespace: arc-systems
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

### Phase 2: Configure Runners

#### 2.1 Update Runner Environment

Add cache server environment variables to `infra/runners/platform-org-runners.yaml`:

```yaml
env:
  # Existing env vars...
  
  # GitHub Actions Cache Server
  - name: ACTIONS_CACHE_URL
    value: "http://gha-cache-server.arc-systems.svc.cluster.local:3000/"
  - name: ACTIONS_RUNTIME_TOKEN
    value: "local"  # Not used but required
```

#### 2.2 Alternative: ARC Scale Set Configuration

For ARC-based runners (`infra/runner-cache/values.yaml`):

```yaml
template:
  spec:
    containers:
      - name: runner
        env:
          - name: ACTIONS_CACHE_URL
            value: "http://gha-cache-server.arc-systems.svc.cluster.local:3000/"
          - name: ACTIONS_RUNTIME_TOKEN
            value: "local"
```

### Phase 3: Update Workflows

#### 3.1 No Workflow Changes Required

The cache server is a **drop-in replacement**. Existing `actions/cache` usage works automatically:

```yaml
# This will work on self-hosted runners with the cache server
- uses: actions/cache@v4
  with:
    path: ~/.cargo
    key: cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      cargo-
```

#### 3.2 Optional: Conditional Caching

For workflows running on both hosted and self-hosted:

```yaml
- uses: actions/cache@v4
  with:
    path: ~/.cargo
    key: cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}
  # Works on both GitHub hosted (uses GitHub cache) and
  # self-hosted (uses local cache server)
```

### Phase 4: Migration Strategy

#### 4.1 Parallel Operation

1. Deploy cache server alongside existing PVC caching
2. Test with a single workflow first
3. Monitor cache hit rates and performance
4. Gradually migrate remaining workflows

#### 4.2 PVC Deprecation

Once validated:

1. Remove PVC volume mounts from runner specs
2. Update environment variables (remove `CARGO_HOME=/cache/cargo` etc.)
3. Keep PVCs for 30 days as fallback
4. Delete PVCs after successful migration

## Storage Backend Options

### Option A: Local Filesystem (Recommended for Start)

```yaml
env:
  STORAGE_DRIVER: local
volumes:
  - name: cache-data
    persistentVolumeClaim:
      claimName: gha-cache-pvc
```

**Pros:** Simple, no additional services  
**Cons:** Single node, limited scalability

### Option B: MinIO (Recommended for Production)

```yaml
env:
  STORAGE_DRIVER: s3
  S3_BUCKET: gha-cache
  S3_REGION: us-east-1
  S3_ENDPOINT: http://minio.storage.svc.cluster.local:9000
  S3_ACCESS_KEY_ID: <from-secret>
  S3_SECRET_ACCESS_KEY: <from-secret>
```

**Pros:** Distributed, scalable, S3-compatible  
**Cons:** Requires MinIO deployment

### Option C: External S3

For cloud deployments or hybrid setups:

```yaml
env:
  STORAGE_DRIVER: s3
  S3_BUCKET: 5dlabs-gha-cache
  S3_REGION: us-west-2
  # Credentials via IRSA or secrets
```

## Cache Management

### Automatic Cleanup

The cache server supports automatic eviction:

```yaml
env:
  CACHE_EVICTION_POLICY: LRU
  CACHE_MAX_SIZE_GB: "80"  # Evict when 80GB reached
```

### Manual Cleanup

```bash
# List caches
curl http://gha-cache-server:3000/cache

# Delete specific cache
curl -X DELETE http://gha-cache-server:3000/cache/{key}
```

## Monitoring

### Prometheus Metrics

The cache server exposes metrics on `/metrics`:

```yaml
# ServiceMonitor for Prometheus
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: gha-cache-server
spec:
  selector:
    matchLabels:
      app: gha-cache-server
  endpoints:
    - port: http
      path: /metrics
```

### Key Metrics

- `gha_cache_hits_total` - Cache hit count
- `gha_cache_misses_total` - Cache miss count
- `gha_cache_size_bytes` - Current cache size
- `gha_cache_evictions_total` - Eviction count

## Security Considerations

### Network Policy

Restrict cache server access to runner namespace:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: gha-cache-server
  namespace: arc-systems
spec:
  podSelector:
    matchLabels:
      app: gha-cache-server
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: arc-runners
      ports:
        - port: 3000
```

### Authentication

For multi-tenant scenarios, enable token authentication:

```yaml
env:
  AUTH_ENABLED: "true"
  AUTH_TOKENS: <from-secret>
```

## Rollback Plan

If issues arise:

1. Remove `ACTIONS_CACHE_URL` from runner configuration
2. Restore PVC volume mounts
3. Restore `CARGO_HOME`, `RUSTUP_HOME` environment variables
4. Runners will fall back to PVC-based caching

## Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| Phase 1 | 1 day | Deploy cache server, verify running |
| Phase 2 | 1 day | Configure runners, test connectivity |
| Phase 3 | 2 days | Test with pilot workflows |
| Phase 4 | 1 week | Gradual migration, monitoring |
| Cleanup | +30 days | Remove deprecated PVCs |

## References

- [falcondev-oss/github-actions-cache-server](https://github.com/falcondev-oss/github-actions-cache-server)
- [Documentation](https://gha-cache-server.falcondev.io/getting-started)
- [GitHub Actions Cache Protocol](https://github.com/actions/toolkit/tree/main/packages/cache)

