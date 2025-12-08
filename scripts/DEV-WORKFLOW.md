# Fast Development Workflow

This guide explains how to iterate quickly on the Talos cluster without waiting for CI.

## Quick Start

```bash
# Build, push, and deploy controller in ~2-3 minutes
./scripts/dev-push.sh controller

# Build and push tools without deploying
./scripts/dev-push.sh --no-deploy tools

# Use a feature branch tag
./scripts/dev-push.sh --tag feature-auth healer

# See all available images
./scripts/dev-push.sh --list
```

## Prerequisites

1. **Docker logged into GHCR**:
   ```bash
   echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
   ```

2. **kubectl configured for Talos cluster**:
   ```bash
   # Should show your Talos nodes
   kubectl get nodes
   ```

3. **Sufficient disk space** for building (~5-10GB per image)

## How It Works

1. **Build**: Uses `Dockerfile.kind` or local Dockerfiles that do full builds inside the container (cross-compiles for linux/amd64)
2. **Push**: Pushes to GHCR with a `dev-$USER` tag (e.g., `ghcr.io/5dlabs/controller:dev-jonathon`)
3. **Deploy**: Patches the deployment to use your dev image and triggers a rollout

## Available Images

| Image | Description | Build Time |
|-------|-------------|------------|
| `controller` | Agent orchestration | ~3-5 min |
| `tools` | MCP server proxy | ~2-3 min |
| `healer` | Self-healing monitor | ~3-5 min |
| `pm` | Project management | ~2-3 min |
| `openmemory` | AI agent memory | ~1-2 min |
| `tweakcn` | Theme editor | ~2-3 min |
| `runtime` | Agent runtime base | ~5-10 min |
| `claude` | Claude CLI image | ~5-10 min |
| `opencode` | OpenCode CLI image | ~3-5 min |
| `dexter` | Dexter CLI image | ~3-5 min |

## Workflow Tips

### Watch Logs During Development

```bash
# In one terminal, watch the deployment
./scripts/dev-push.sh controller

# In another terminal, stream logs
kubectl logs -f deployment/controller -n cto
```

### Port Forward for Testing

```bash
# Controller API
kubectl port-forward deployment/controller -n cto 8080:8080

# Tools server
kubectl port-forward deployment/tools -n cto 3000:3000

# Healer
kubectl port-forward deployment/healer -n cto 8081:8080
```

### Quick Iteration Loop

```bash
# 1. Make code changes
vim crates/controller/src/main.rs

# 2. Build, push, deploy
./scripts/dev-push.sh controller

# 3. Test your changes
curl http://localhost:8080/health

# 4. Check logs if something's wrong
kubectl logs deployment/controller -n cto --tail=50
```

### Revert to Production

When you're done testing, revert to the production image:

```bash
# Option 1: Force ArgoCD sync (recommended)
argocd app sync cto --force

# Option 2: Manual patch back to latest
kubectl set image deployment/controller controller=ghcr.io/5dlabs/controller:latest -n cto
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `REGISTRY` | `ghcr.io` | Container registry |
| `ORG` | `5dlabs` | Registry organization |
| `NAMESPACE` | `cto` | Kubernetes namespace |
| `DEV_TAG` | `dev-$USER` | Default image tag |

## Troubleshooting

### Build Fails with Cargo Errors

The multi-stage Dockerfiles compile Rust inside the container. If you hit dependency issues:

```bash
# Clear Docker build cache
docker builder prune -f

# Rebuild without cache
docker build --no-cache --platform linux/amd64 ...
```

### Push Fails with Auth Errors

```bash
# Re-authenticate to GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u $(git config user.name) --password-stdin
```

### Deployment Doesn't Update

```bash
# Force a pod restart
kubectl rollout restart deployment/controller -n cto

# Check if image was pulled
kubectl describe pod -l app.kubernetes.io/name=controller -n cto | grep Image
```

### Image Pull Errors on Talos

If the cluster can't pull your dev image:

1. Verify the image exists: `docker manifest inspect ghcr.io/5dlabs/controller:dev-yourname`
2. Check image pull secrets: `kubectl get secrets ghcr-secret -n cto -o yaml`
3. Ensure the secret has access to the tag you pushed

## Comparison: CI vs Dev Push

| Aspect | CI Pipeline | Dev Push |
|--------|-------------|----------|
| Build time | 5-15 min (queue + build) | 2-5 min |
| Trigger | Git push | Manual |
| Testing | Full test suite | Your local tests |
| Image tag | `latest`, `v1.2.3` | `dev-$USER` |
| Deployment | Automatic via ArgoCD | Immediate patch |
| Rollback | Previous version | `argocd sync --force` |

## Advanced: Multiple Dev Images

If multiple developers are testing simultaneously:

```bash
# Each developer uses their own tag
DEV_TAG="dev-alice" ./scripts/dev-push.sh controller  # Alice
DEV_TAG="dev-bob" ./scripts/dev-push.sh controller    # Bob

# Or use feature names
./scripts/dev-push.sh --tag feature-auth controller
./scripts/dev-push.sh --tag bugfix-123 controller
```

## See Also

- [Talos Cluster Setup](../infra/talos/README.md)
- [Kind Local Development](../tests/kind/README.md)
- [CI Workflows](../.github/workflows/)

