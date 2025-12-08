# Fast Development Workflow

This guide explains how to iterate quickly on the Talos cluster without waiting for CI.

## Two Options

| Script | Network | Speed | Best For |
|--------|---------|-------|----------|
| `dev-load.sh` | Local only | ⚡ Fastest | Day-to-day iteration |
| `dev-push.sh` | Internet (GHCR) | Fast | Sharing with team |

## Option 1: Local Registry (Recommended)

Build and push to an in-cluster registry. No internet required!

### One-Time Setup

```bash
# Deploy a local registry to your Talos cluster
./scripts/dev-load.sh --setup

# Configure Docker to trust the insecure registry
# Add to Docker Desktop → Settings → Docker Engine:
# {"insecure-registries": ["192.168.1.77:30500"]}
# Then restart Docker
```

### Usage

```bash
# Build and deploy controller (~2 min after first build)
./scripts/dev-load.sh controller

# Build and push without deploying
./scripts/dev-load.sh --no-deploy tools

# See all available images
./scripts/dev-load.sh --list
```

### How It Works

1. **Build**: Compiles locally for linux/amd64
2. **Push**: To in-cluster registry at `<node-ip>:30500` (fast local network)
3. **Deploy**: Patches deployment and triggers rollout

---

## Option 2: GHCR Push

Build locally and push to GitHub Container Registry. Useful when sharing with team.

### Prerequisites

```bash
# Login to GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
```

### Usage

```bash
# Build and deploy
./scripts/dev-push.sh controller

# Use a feature branch tag
./scripts/dev-push.sh --tag feature-auth healer
```

### How It Works

1. **Build**: Compiles locally for linux/amd64
2. **Push**: To GHCR with `dev-$USER` tag
3. **Deploy**: Patches deployment and triggers rollout

---

## Available Images

| Image | Description | Build Time |
|-------|-------------|------------|
| `controller` | Agent orchestration | ~3-5 min (first), ~1-2 min (cached) |
| `tools` | MCP server proxy | ~2-3 min |
| `healer` | Self-healing monitor | ~3-5 min |
| `pm` | Project management | ~2-3 min |
| `openmemory` | AI agent memory | ~1-2 min |
| `tweakcn` | Theme editor | ~2-3 min |
| `runtime` | Agent runtime base | ~5-10 min |
| `claude` | Claude CLI image | ~5-10 min |
| `opencode` | OpenCode CLI image | ~3-5 min |
| `dexter` | Dexter CLI image | ~3-5 min |

## Quick Iteration Loop

```bash
# 1. Make code changes
vim crates/controller/src/main.rs

# 2. Build and deploy (~1-2 min with cache)
./scripts/dev-load.sh controller

# 3. Watch logs
kubectl logs -f deployment/controller -n cto

# 4. Repeat!
```

## Port Forward for Testing

```bash
# Controller API
kubectl port-forward deployment/controller -n cto 8080:8080

# Tools server
kubectl port-forward deployment/tools -n cto 3000:3000

# Healer
kubectl port-forward deployment/healer -n cto 8081:8080
```

## Revert to Production

When you're done testing:

```bash
# Force ArgoCD sync (recommended)
argocd app sync cto --force

# Or manual patch
kubectl set image deployment/controller controller=ghcr.io/5dlabs/controller:latest -n cto
```

## Troubleshooting

### "insecure registry" Error

Docker doesn't trust the in-cluster registry by default:

```bash
# Get your node IP
kubectl get nodes -o wide

# Add to Docker Desktop → Settings → Docker Engine:
{"insecure-registries": ["<NODE_IP>:30500"]}

# Restart Docker
```

### Build Fails with Cargo Errors

```bash
# Clear Docker build cache
docker builder prune -f

# Rebuild without cache
docker build --no-cache --platform linux/amd64 ...
```

### Deployment Doesn't Update

```bash
# Force a pod restart
kubectl rollout restart deployment/controller -n cto

# Check if image was pulled
kubectl describe pod -l app.kubernetes.io/name=controller -n cto | grep Image
```

### Registry Not Reachable

```bash
# Check registry is running
kubectl get pods -n registry

# Test connectivity from your machine
curl http://<NODE_IP>:30500/v2/_catalog
```

## Comparison

| Aspect | CI Pipeline | dev-load.sh | dev-push.sh |
|--------|-------------|-------------|-------------|
| Network | Internet | Local only | Internet |
| Time | 5-15 min | 1-3 min | 2-5 min |
| Sharing | Everyone | Just you | Team |
| Best for | Production | Fast iteration | Team testing |

## See Also

- [Talos Cluster Setup](../infra/talos/README.md)
- [Kind Local Development](../tests/kind/README.md)
- [CI Workflows](../.github/workflows/)

