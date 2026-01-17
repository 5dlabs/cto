# Fast Dev Image Builds

This guide explains how to rapidly iterate on agent images (runtime, claude, etc.) without waiting for GitHub Actions CI.

## Overview

The standard CI workflow takes 10-15+ minutes:

```
Tag push → binaries-release.yaml (build all platforms) → agents-build.yaml (runtime + agents)
```

The dev workflow takes ~2-3 minutes:

```
cargo-zigbuild (cross-compile) → Docker overlay image → Push to GHCR
```

## Quick Start

```bash
# One-time setup: Install cross-compilation tools
just install-cross-tools

# Authenticate with GHCR (if not already done)
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Build and push dev image with updated intake
just dev-claude-image
```

## How It Works

### 1. Cross-Compilation (cargo-zigbuild)

Instead of building inside Docker with emulation (slow on Mac), we use `cargo-zigbuild` to cross-compile Rust binaries directly on macOS:

```bash
cargo zigbuild --release --target x86_64-unknown-linux-gnu -p intake
```

This produces a Linux x86_64 binary in ~1-2 minutes on Apple Silicon.

### 2. Overlay Image

Instead of rebuilding the entire runtime image, we create a minimal overlay:

```dockerfile
FROM ghcr.io/5dlabs/claude:latest
USER root
COPY intake /usr/local/bin/intake
RUN chmod +x /usr/local/bin/intake
USER node
```

This image builds in seconds since it only adds one layer.

### 3. Push to GHCR

The dev image is pushed to `ghcr.io/5dlabs/<image>:dev`:

- `ghcr.io/5dlabs/runtime:dev`
- `ghcr.io/5dlabs/claude:dev`
- etc.

## Available Commands

| Command | Description |
|---------|-------------|
| `just dev-runtime-image` | Build runtime with local intake → push to GHCR |
| `just dev-claude-image` | Build Claude image with local intake → push to GHCR |
| `just dev-image-local` | Build locally without pushing (testing) |
| `just dev-runtime-all` | Build with all binaries (intake + pm-activity) |
| `just install-cross-tools` | Install cargo-zigbuild |

## Script Options

```bash
./scripts/build-dev-image.sh --help

# Examples:
./scripts/build-dev-image.sh --binary intake --image runtime --push
./scripts/build-dev-image.sh --binary all --image claude --push
./scripts/build-dev-image.sh --binary intake --image claude --tag my-feature --push
```

### Parameters

| Parameter | Values | Default | Description |
|-----------|--------|---------|-------------|
| `--binary` | `intake`, `pm-activity`, `all` | `intake` | Which binary to build |
| `--image` | `runtime`, `claude`, `cursor`, `codex` | `runtime` | Base image to overlay |
| `--tag` | any string | `dev` | Docker tag for the image |
| `--push` | flag | false | Push to GHCR after building |

## Using the Dev Image

### Option A: Direct Container Test

```bash
# Test locally (shows warning about platform mismatch on Mac, but works)
docker run --rm ghcr.io/5dlabs/claude:dev intake --version
```

### Option B: Cluster Deployment

#### For In-Cluster Controller

Update the controller's configmap to use the dev tag:

```bash
# Get current config
kubectl get configmap task-controller-config -n cto -o jsonpath='{.data.config\.yaml}' > /tmp/config.yaml

# Modify claude tag to "dev"
yq e '.agent.cliImages.claude.tag = "dev"' -i /tmp/config.yaml

# Apply
kubectl create configmap task-controller-config -n cto \
  --from-file=config.yaml=/tmp/config.yaml \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart controller
kubectl rollout restart deployment cto-controller -n cto
```

To restore:

```bash
yq e '.agent.cliImages.claude.tag = "latest"' -i /tmp/config.yaml
kubectl create configmap task-controller-config -n cto \
  --from-file=config.yaml=/tmp/config.yaml \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl rollout restart deployment cto-controller -n cto
```

#### For Local Controller (launchd)

Update your local `cto-config.json` or controller config file to use the dev image.

### Option C: cto-config.json Override

For specific Play workflows, you can override the agent image in your `cto-config.json`:

```json
{
  "defaults": {
    "play": {
      "agentImage": "ghcr.io/5dlabs/claude:dev"
    }
  }
}
```

## Workflow Example

Here's a complete example of iterating on the intake binary:

```bash
# 1. Make changes to intake
vim crates/intake/src/lib.rs

# 2. Test locally
cargo run --bin intake -- --version

# 3. Build and push dev image
just dev-claude-image

# 4. Verify image
docker run --rm ghcr.io/5dlabs/claude:dev intake --version

# 5. Test in cluster (if using in-cluster controller)
# Update configmap as shown above, then create a test CodeRun

# 6. When satisfied, create a proper release
git tag -a v0.2.13 -m "Release v0.2.13"
git push origin v0.2.13
# This triggers the full CI pipeline
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Development Flow                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   macOS (Apple Silicon)                                              │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  cargo-zigbuild                                              │   │
│   │  ┌─────────────┐       ┌─────────────────────────────────┐  │   │
│   │  │ Rust Source │ ───── │ x86_64-unknown-linux-gnu binary │  │   │
│   │  └─────────────┘       └─────────────────────────────────┘  │   │
│   │         ~1-2 minutes on M1/M2                                │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                               │                                      │
│                               ▼                                      │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  Docker Buildx                                               │   │
│   │  ┌─────────────────────┐    ┌───────────────────────────┐   │   │
│   │  │ FROM claude:latest  │ +  │ COPY intake /usr/local/bin│   │   │
│   │  └─────────────────────┘    └───────────────────────────┘   │   │
│   │         ~10 seconds (overlay only)                           │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                               │                                      │
│                               ▼                                      │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  GHCR Push                                                   │   │
│   │  ghcr.io/5dlabs/claude:dev                                   │   │
│   │         ~30 seconds (only new layer)                         │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Kubernetes Cluster                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   task-controller-config:                                            │
│     agent.cliImages.claude.tag: "dev"  ← Update to test              │
│                                                                      │
│   CodeRun Jobs pull ghcr.io/5dlabs/claude:dev                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### cargo-zigbuild not found

```bash
cargo install cargo-zigbuild
```

### GHCR authentication failed

```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
```

### Image not pulling in cluster

Check image pull secrets:

```bash
kubectl get secret ghcr-secret -n cto -o yaml
```

### Controller not using new image

1. Verify configmap was updated:
   ```bash
   kubectl get configmap task-controller-config -n cto -o jsonpath='{.data.config\.yaml}' | yq e '.agent.cliImages.claude' -
   ```

2. Restart controller:
   ```bash
   kubectl rollout restart deployment cto-controller -n cto
   ```

3. Check controller logs:
   ```bash
   kubectl logs -f deployment/cto-controller -n cto
   ```

## Comparison with Full CI

| Aspect | Dev Build | Full CI |
|--------|-----------|---------|
| Time | ~2-3 minutes | ~15+ minutes |
| Platforms | linux/amd64 only | linux/amd64, darwin, windows |
| Binaries | Selected (e.g., intake) | All binaries |
| Testing | Manual verification | Automated tests |
| Use case | Rapid iteration | Production releases |

## Best Practices

1. **Always test locally first** before pushing dev images
2. **Use descriptive tags** for feature branches: `--tag my-feature`
3. **Clean up dev images** when done testing
4. **Don't use dev images in production** - always go through full CI for releases
5. **Document what changed** in the dev image for teammates
