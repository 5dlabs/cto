# Fast Development Workflow

This guide explains how to iterate quickly without waiting for CI.

## Daily Market Sync Baseline

Use this baseline when running the model-catalog sync workflow so generated
catalog diffs are always based on the remote default branch.

```bash
# Start from the repo root
git fetch --prune origin

# Resolve the remote default branch (origin/HEAD -> origin/main, etc)
DEFAULT_BRANCH="$(git symbolic-ref --short refs/remotes/origin/HEAD | sed 's|^origin/||')"

# Update local default branch from origin
git switch "$DEFAULT_BRANCH"
git pull --ff-only origin "$DEFAULT_BRANCH"

# Create a short-lived sync branch and run the sync script
git switch -c "chore/model-catalog-sync-$(date +%Y%m%d)"
./scripts/sync-model-catalog.sh
```

The sync PR should include model-catalog artifacts and derived config files
only. Avoid bundling unrelated infrastructure or application changes.

## Development Options

| Method | Network | Speed | Best For |
|--------|---------|-------|----------|
| **Native Local** (`just` + `bacon`) | Cluster API | ⚡⚡ Fastest | Day-to-day iteration |
| `dev-load.sh` + Helm toggle | Local only | ⚡ Fast | Testing in real pods |
| `dev-push.sh` | Internet (GHCR) | Fast | Sharing with team |
| Tilt | Local/Internet | Medium | Full stack testing |

---

## Option 0: Native Local Development (Recommended)

Run services directly on your machine as native binaries, connecting to the real cluster via kubeconfig. This is the fastest iteration loop (~5-15s incremental builds vs ~1-3 min Docker builds).

### Prerequisites

```bash
# Install development tools
cargo install just bacon

# Or using just itself
just install-tools
```

### One-Time Setup

```bash
# 1. Set up your environment variables
./scripts/sync-secrets-for-dev.sh   # Fetches from 1Password
# OR
cp env.template .env.local          # Manual setup
# Edit .env.local with your values

# 2. Source the environment
source .env.local

# 3. Verify setup
just env-info
./scripts/dev-local.sh --check
```

### Quick Start

```bash
# Source your environment
source .env.local

# Start all services
just dev

# Or start specific services
just dev-pm          # PM server only
just dev-controller  # Controller only
just dev-tools       # Tools server only
just dev-healer      # Healer server only
```

### Using Bacon (File Watching)

Bacon watches your code and rebuilds automatically:

```bash
# In one terminal - run a service with watching
bacon run-pm         # Watch and run PM server
bacon run-controller # Watch and run controller
bacon run-tools      # Watch and run tools server
bacon run-healer     # Watch and run healer

# In another terminal - check/lint while coding
bacon                # cargo check (default)
bacon clippy         # clippy with pedantic
bacon test           # run tests
```

### Available Just Commands

```bash
just --list          # Show all commands

# Build
just build           # Build all binaries (debug)
just build-release   # Build all binaries (release)

# Check & Lint
just check           # cargo check
just clippy          # clippy pedantic (required before push)
just fmt             # Format code
just pre-push        # Run all pre-push checks

# Test
just test            # Run all tests
just test-verbose    # Tests with output

# Development
just dev             # Start all services
just dev-pm          # Start PM server
just dev-controller  # Start controller
just dev-tools       # Start tools server
just dev-healer      # Start healer

# CLIs
just intake <args>   # Run intake CLI
just research <args> # Run research CLI
just healer <args>   # Run healer CLI
just mcp             # Run MCP server
```

### Services & Ports

| Service | Port | Binary | Notes |
|---------|------|--------|-------|
| PM Server | 8081 | `pm-server` | Linear webhooks |
| Controller | 8080 | `agent_controller` | CodeRun CRD controller |
| Tools | 3000 | `tools-server` | MCP proxy |
| Healer | 8080 | `healer server` | Self-healing monitor |

### Environment Variables

All required secrets are documented in `env.template`. Key variables:

| Variable | Service | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | All | Claude API key |
| `LINEAR_OAUTH_TOKEN` | PM | Linear API access |
| `GITHUB_TOKEN` | Multiple | GitHub operations |
| `NAMESPACE` | All | Kubernetes namespace (default: `cto`) |
| `AGENT_TEMPLATES_PATH` | Controller | Path to templates |

### Comparison: Local vs Docker

| Metric | Docker/Tilt | Native Local |
|--------|-------------|--------------|
| Initial build | ~3-5 min | ~1-2 min |
| Incremental rebuild | ~30-60s | ~5-15s |
| Hot reload | No | Yes (bacon) |
| Feedback loop | Build → Push → Deploy → Test | Save → Rebuild → Test |
| Real cluster | Yes (in pods) | Yes (via kubeconfig) |

---

## Option 1: Local Registry (Docker-based)

Build and push to an in-cluster registry. Good for testing in real pods.

### Quick Reference: Helm Dev Registry Toggle

The CTO Helm chart supports a `global.devRegistry` toggle to switch between GHCR and local registry:

```bash
# Enable local registry for all components
helm upgrade cto ./infra/charts/cto -n cto \
  --set global.devRegistry.enabled=true \
  --set global.devRegistry.url=192.168.1.77:30500

# Switch back to GHCR
helm upgrade cto ./infra/charts/cto -n cto \
  --set global.devRegistry.enabled=false

# Use a specific tag for one component
helm upgrade cto ./infra/charts/cto -n cto \
  --set global.devRegistry.enabled=true \
  --set global.devRegistry.componentTags.controller=feature-branch
```

### Local Registry Details

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

### Using with Helm (Recommended)

Instead of patching deployments directly, use the Helm `devRegistry` toggle for cleaner switching:

```bash
# 1. Build and push to local registry (--no-deploy)
./scripts/dev-load.sh --no-deploy controller
./scripts/dev-load.sh --no-deploy tools
./scripts/dev-load.sh --no-deploy healer

# 2. Enable dev registry in Helm (applies to all components)
helm upgrade cto ./infra/charts/cto -n cto \
  --reuse-values \
  --set global.devRegistry.enabled=true \
  --set global.devRegistry.url=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}'):30500

# 3. Switch back to GHCR when done
helm upgrade cto ./infra/charts/cto -n cto \
  --reuse-values \
  --set global.devRegistry.enabled=false
```

### How It Works

1. **Build**: Compiles locally for linux/amd64
2. **Push**: To in-cluster registry at `<node-ip>:30500` (fast local network)
3. **Deploy**: Patches deployment and triggers rollout

---

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

## Helm Dev Registry Configuration

The CTO Helm chart has a `global.devRegistry` section that controls where images are pulled from:

### Values

```yaml
global:
  devRegistry:
    enabled: false           # Toggle: true = local registry, false = GHCR
    url: "192.168.1.77:30500" # Your node IP + NodePort
    tag: "dev-local"         # Default tag for dev images
    pullPolicy: Always       # Always pull to get latest builds
    componentTags: {}        # Override tags per component
```

### Examples

```bash
# Enable dev registry for all components
helm upgrade cto ./infra/charts/cto -n cto --reuse-values \
  --set global.devRegistry.enabled=true

# Use custom tags for specific components
helm upgrade cto ./infra/charts/cto -n cto --reuse-values \
  --set global.devRegistry.enabled=true \
  --set global.devRegistry.componentTags.controller=feature-x \
  --set global.devRegistry.componentTags.healer=debug

# Switch a single component to dev while others stay on GHCR
# (Build with custom tag, leave devRegistry disabled)
./scripts/dev-load.sh --tag my-fix controller
kubectl set image deployment/controller controller=192.168.1.77:30500/controller:my-fix -n cto
```

### Supported Components

| Component | Key | Notes |
|-----------|-----|-------|
| controller | `controller` | Agent orchestration |
| pm | `pm` | Project management |
| healer | `healer` | Self-healing monitor |
| tools | `tools` | MCP proxy server |
| research | `research` | Twitter research pipeline |
| openmemory | `openmemory` | AI agent memory |
| tweakcn | `tweakcn` | Theme editor |

---

## Comparison

| Aspect | CI Pipeline | Native Local | dev-load.sh | dev-push.sh |
|--------|-------------|--------------|-------------|-------------|
| Network | Internet | Cluster API | Local only | Internet |
| Build Time | 5-15 min | 5-15s (incr) | 1-3 min | 2-5 min |
| Rebuild Time | 5-15 min | 5-15s | 30-60s | 1-2 min |
| Sharing | Everyone | Just you | Just you | Team |
| Real Pods | Yes | No (native) | Yes | Yes |
| Best for | Production | Day-to-day | Pod testing | Team testing |

## See Also

- **Local Development Files:**
  - [`justfile`](../justfile) - Command runner configuration
  - [`bacon.toml`](../bacon.toml) - File watcher configuration
  - [`env.template`](../env.template) - Environment variable template
  - [`scripts/sync-secrets-for-dev.sh`](./sync-secrets-for-dev.sh) - 1Password secret sync
  - [`scripts/dev-local.sh`](./dev-local.sh) - Service orchestration script
- **Infrastructure:**
  - [Talos Cluster Setup](../infra/talos/README.md)
  - [Kind Local Development](../tests/kind/README.md)
  - [CI Workflows](../.github/workflows/)

