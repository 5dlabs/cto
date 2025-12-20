# Depot.dev Integration for Faster Docker Builds

This document explains how to set up [Depot.dev](https://depot.dev) for accelerating Docker image builds in GitHub Actions.

## Current Configuration

| Setting | Value |
|---------|-------|
| **Organization** | 5D Labs (`std7z6mpmx`) |
| **Project** | cto-platform (`w9np5s6src`) |
| **Region** | us-east-1 |
| **Cache Size** | 100 GB |

Credentials are stored in 1Password: **"Depot.dev API Token - CTO Platform"**

## Why Depot?

Depot provides **remote Docker builds** with several advantages over standard GitHub Actions:

| Feature | GitHub Actions (Standard) | Depot |
|---------|--------------------------|-------|
| **Build Cache** | Per-workflow, cleared often | Persistent across all builds |
| **Builder Resources** | 2 CPU / 7GB RAM | 16 CPU / 32GB RAM (default) |
| **Multi-platform** | QEMU emulation (slow) | Native ARM + x86 |
| **Push Speed** | Through runner network | Direct from Depot to registry |
| **Cold Build** | ~5-10 minutes | ~1-3 minutes (with cache) |

## Setup Instructions

### 1. OIDC Trust Relationship (Required for GitHub Actions)

OIDC authentication is the most secure option—no static tokens to manage.

**⚠️ This step must be done in the Depot web UI:**

1. Go to: https://depot.dev/orgs/std7z6mpmx/projects/w9np5s6src/settings
2. Scroll to **Trust Relationships**
3. Click **Add Trust Relationship**
4. Select **GitHub Actions**
5. Configure:
   - **Repository**: `5dlabs/cto`
   - **Environment**: Leave empty (allows all branches/environments)
6. Save

### 2. Configuration Files

The project is already configured via `depot.json` in the repo root:

```json
{
  "$schema": "https://depot.dev/schema.json",
  "id": "w9np5s6src"
}
```

**Alternative: Using GitHub Secrets** (if you prefer not to commit the project ID)

1. Go to GitHub repo → Settings → Secrets and Variables → Actions
2. Add a new secret:
   - Name: `DEPOT_PROJECT_ID`
   - Value: `w9np5s6src`

### 5. Run the POC Workflow

The POC workflow at `.github/workflows/controller-ci-depot.yaml` runs both:
- Standard `docker/build-push-action` build
- Depot `depot/build-push-action` build

This allows you to compare timing in the GitHub Actions summary.

Trigger the workflow:
```bash
gh workflow run "Controller CI (Depot POC)" --ref main
```

Or push changes to the workflow file to trigger automatically.

## Expected Results

### First Build (Cold Cache)
- Standard: ~2-3 minutes (simple Dockerfile)
- Depot: ~1-2 minutes

### Subsequent Builds (Warm Cache)
- Standard: ~1-2 minutes (if GHA cache hits)
- Depot: ~30-60 seconds (persistent cache always available)

The real benefits become apparent with:
- More complex Dockerfiles (multi-stage builds)
- Multi-platform builds (linux/amd64 + linux/arm64)
- Higher build frequency (cache stays warm)

## Integrating into Production Workflow

Once validated, integrate Depot into the main `controller-ci.yaml`:

### Option 1: Replace the docker-build-push composite action

Update `.github/actions/docker-build-push/action.yaml` to use Depot:

```yaml
- name: Build and push Docker image
  uses: depot/build-push-action@v1
  with:
    project: ${{ inputs.depot-project-id }}
    context: ${{ inputs.context }}
    file: ${{ inputs.dockerfile }}
    platforms: ${{ inputs.platforms }}
    push: ${{ inputs.push }}
    tags: ${{ steps.meta.outputs.tags }}
```

### Option 2: Use Depot directly in workflows

Replace the `build-and-push` job with:

```yaml
build-and-push:
  runs-on: ubuntu-latest  # Depot builds remotely
  permissions:
    contents: read
    packages: write
    id-token: write  # Required for OIDC
  steps:
    - uses: actions/checkout@v4
    
    - uses: depot/setup-action@v1
    
    - uses: docker/login-action@v3
      with:
        registry: ghcr.io
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - uses: depot/build-push-action@v1
      with:
        context: .
        file: ./infra/images/controller/Dockerfile
        push: true
        tags: ghcr.io/5dlabs/controller:latest
```

## Multi-Platform Builds

One of Depot's killer features is native multi-platform builds:

```yaml
- uses: depot/build-push-action@v1
  with:
    platforms: linux/amd64,linux/arm64
    # ... other options
```

This builds both architectures **natively** (no QEMU emulation), which is:
- 5-10x faster than emulated builds
- Produces better-optimized binaries

## Cost Considerations

Depot pricing (as of 2024):

| Plan | Cost | Build Minutes | Notes |
|------|------|--------------|-------|
| Developer | $20/month | 500 minutes | 1 user |
| Startup | $200/month | 5,000 minutes | Unlimited users |
| Business | Custom | Custom | Enterprise features |

For comparison:
- GitHub-hosted runners: Free tier + $0.008/minute after
- Self-hosted k8s-runner: Infrastructure cost only

Given we already have self-hosted runners, Depot makes most sense for:
1. Docker builds specifically (where persistent cache helps most)
2. Multi-platform builds (avoid QEMU)
3. When GitHub-hosted runner capacity is needed

## Troubleshooting

### Build fails with "project not found"
- Verify `depot.json` has correct project ID
- Check OIDC trust relationship is configured for `5dlabs/cto`

### Build fails with "authentication error"
- Ensure `id-token: write` permission is in workflow
- Verify trust relationship matches repo/branch

### Cache not being used
- Check Depot dashboard for cache statistics
- First build after creating project will be cold

## References

- [Depot Documentation](https://depot.dev/docs)
- [GitHub Actions Integration](https://depot.dev/docs/container-builds/integrations/github-actions)
- [OIDC Authentication](https://depot.dev/docs/cli/authentication#oidc-trust-relationships)
- [Depot Pricing](https://depot.dev/pricing)
