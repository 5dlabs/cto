# Continuous Delivery Pipeline

This document describes the automated continuous delivery pipeline for the CTO platform.

## Overview

The CD pipeline automatically builds, versions, and deploys changes with minimal human intervention:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. Developer merges PR with conventional commits                           │
│                              ↓                                              │
│  2. Release Please creates/updates a Release PR                             │
│                              ↓                                              │
│  3. Merge Release PR → GitHub Release + Tag created (v0.2.1)               │
│                              ↓                                              │
│  4. binaries-release workflow builds and pushes images                      │
│                              ↓                                              │
│  5. ArgoCD Image Updater detects new tag in ghcr.io                        │
│                              ↓                                              │
│  6. Image Updater updates Application parameters                            │
│                              ↓                                              │
│  7. ArgoCD syncs → Rolling deployment with new image                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Release Please

[Release Please](https://github.com/googleapis/release-please) automates versioning based on [Conventional Commits](https://www.conventionalcommits.org/).

**Commit message prefixes:**

| Prefix | Version Bump | Example |
|--------|--------------|---------|
| `feat:` | Minor (0.2.0 → 0.3.0) | `feat(controller): add task retry logic` |
| `fix:` | Patch (0.2.0 → 0.2.1) | `fix: resolve nil pointer in reconciler` |
| `feat!:` or `BREAKING CHANGE:` | Major (0.2.0 → 1.0.0) | `feat!: redesign API` |
| `chore:`, `docs:`, `refactor:` | No bump | `chore: update dependencies` |

**Configuration files:**
- `.github/workflows/release-please.yaml` - The workflow
- `release-please-config.json` - Release configuration
- `.release-please-manifest.json` - Current version tracking

### 2. ArgoCD Image Updater

[ArgoCD Image Updater](https://argocd-image-updater.readthedocs.io/) watches container registries and updates ArgoCD Applications when new images are available.

**How it works:**
1. Periodically polls ghcr.io for new image tags
2. Compares tags against the configured strategy (semver)
3. Updates the Helm `image.tag` parameter in the ArgoCD Application
4. ArgoCD detects the change and syncs (rolls out new pods)

**Application annotations:**
```yaml
annotations:
  # Image to watch
  argocd-image-updater.argoproj.io/image-list: controller=ghcr.io/5dlabs/controller
  # Strategy: semver picks highest semantic version
  argocd-image-updater.argoproj.io/controller.update-strategy: semver
  # Only allow v*.*.* tags
  argocd-image-updater.argoproj.io/controller.allow-tags: regexp:^v[0-9]+\.[0-9]+\.[0-9]+$
  # Update method: argocd (parameter override) or git (commit to repo)
  argocd-image-updater.argoproj.io/write-back-method: argocd
  # Which Helm parameter to update
  argocd-image-updater.argoproj.io/controller.helm.image-tag: image.tag
```

## Setup

### Prerequisites

1. ArgoCD installed and configured
2. GitHub Actions workflows enabled
3. GHCR (GitHub Container Registry) access

### Setting up GHCR Credentials for Image Updater

The Image Updater needs credentials to access private images in ghcr.io:

```bash
# Create a GitHub Personal Access Token (PAT) with read:packages scope
# Then create the secret in the argocd namespace

kubectl create secret docker-registry ghcr-credentials \
  --namespace argocd \
  --docker-server=ghcr.io \
  --docker-username=YOUR_GITHUB_USERNAME \
  --docker-password=YOUR_GITHUB_PAT \
  --docker-email=your-email@example.com
```

Alternatively, use the existing `ghcr-pull-secret` if configured:
```yaml
credentials: pullsecret:argocd/ghcr-pull-secret
```

### Deploying Image Updater

The Image Updater is deployed via ArgoCD:

```bash
# Sync the platform-apps to deploy Image Updater
kubectl patch application platform-apps -n argocd --type merge \
  -p '{"operation":{"sync":{"prune":true}}}'
```

Or apply manually:
```bash
kubectl apply -f infra/gitops/applications/argocd-image-updater.yaml
```

## Workflow

### Making a Release

1. **Develop and merge PRs** with conventional commit messages:
   ```bash
   git commit -m "feat(controller): add support for task cancellation"
   git commit -m "fix(heal): resolve alert deduplication bug"
   ```

2. **Release Please creates a Release PR** automatically:
   - Title: `chore: release 0.3.0`
   - Contains version bumps in `Cargo.toml` and `Chart.yaml`
   - Includes auto-generated `CHANGELOG.md`

3. **Review and merge the Release PR**:
   - This creates a GitHub Release with tag `v0.3.0`
   - Triggers the `binaries-release` workflow

4. **Automatic deployment**:
   - Images are built and pushed with the new tag
   - Image Updater detects the new tag
   - ArgoCD rolls out the new version

### Manual Release (if needed)

```bash
# Create a tag manually
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin v0.3.0

# Or trigger release workflow manually
gh workflow run binaries-release.yaml -f version=0.3.0
```

## Monitoring

### Check Image Updater logs

```bash
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-image-updater -f
```

### Check which images are being tracked

```bash
kubectl get applications -n argocd -o yaml | \
  grep -A5 "argocd-image-updater.argoproj.io/image-list"
```

### Force Image Updater to check now

```bash
kubectl rollout restart deployment argocd-image-updater -n argocd
```

## Troubleshooting

### Images not updating

1. **Check Image Updater logs** for errors
2. **Verify the image exists** in the registry:
   ```bash
   docker manifest inspect ghcr.io/5dlabs/controller:v0.3.0
   ```
3. **Check tag pattern** matches `allow-tags` regex
4. **Verify credentials** are correct and have `read:packages` scope

### Release Please not creating PRs

1. **Check workflow runs** in GitHub Actions
2. **Verify commit messages** follow Conventional Commits format
3. **Check the manifest file** has the correct current version

### ArgoCD not syncing

1. **Check Application status**:
   ```bash
   kubectl get application cto-controller -n argocd -o yaml
   ```
2. **Force sync**:
   ```bash
   kubectl patch application cto-controller -n argocd --type merge \
     -p '{"operation":{"sync":{}}}'
   ```

## Best Practices

1. **Always use conventional commits** - This ensures proper version bumping
2. **Don't skip the Release PR** - Review the changelog before releasing
3. **Use semver tags** - Avoid `latest` tag for production deployments
4. **Monitor the pipeline** - Set up alerts for failed workflows
5. **Keep Image Updater credentials rotated** - Use short-lived tokens when possible
