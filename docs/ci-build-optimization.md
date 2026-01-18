# CI Build Optimization Guide

This document describes the optimizations applied to the agent image build pipeline and best practices for maintaining fast, reliable builds.

## Overview

The agent image build pipeline builds Docker images for AI coding assistants (Claude, Codex, Cursor, etc.) that run on the CTO platform. Build performance is critical for developer iteration speed.

### Performance Targets

| Metric | Target | Achieved |
|--------|--------|----------|
| Full pipeline | <15 min | ~14 min |
| Runtime build | <5 min | ~2.5 min |
| Claude agent | <5 min | ~3.5 min |
| Success rate | 100% | 100% |

## Architecture

```
┌─────────────────┐
│  build-runtime  │  Build base image (if changed)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   warm-cache    │  Build cursor first (populates registry cache)
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      build-agents (parallel)                         │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────┐  │
│  │  claude   │ │   codex   │ │   code    │ │  factory  │ │ etc.  │  │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Optimizations

### 1. Cache Warming Phase

The most impactful optimization. Before parallel agent builds start, a dedicated `warm-cache` job builds the simplest agent (cursor) first. This:

- Populates the registry buildcache with all base image layers
- Ensures subsequent agent builds get cache hits
- Reduced Claude build time from 31 min to 3.5 min

**Key insight**: When multiple agents start simultaneously, they compete for cache. The first one to request layers populates the cache, others wait or re-download. By serializing the first build, all others benefit.

### 2. Registry BuildCache

All builds use BuildKit's registry cache backend:

```yaml
cache-from: |
  type=registry,ref=ghcr.io/5dlabs/runtime:buildcache
  type=registry,ref=ghcr.io/5dlabs/${{ matrix.image }}:buildcache
cache-to: |
  type=registry,ref=ghcr.io/5dlabs/${{ matrix.image }}:buildcache,mode=max
```

Benefits:
- Persists across workflow runs (unlike GHA cache which has 10GB limit)
- Shared between all runners
- Contains complete layer chain

### 3. Simplified Dockerfile Syntax

Agent Dockerfiles use `# syntax=docker/dockerfile:1` (standard) instead of `# syntax=docker/dockerfile:1.7-labs` (experimental).

**Why**: The labs syntax requires downloading a separate frontend image (~12MB) before each build starts. Standard syntax starts immediately.

**Exception**: The runtime Dockerfile uses 1.7-labs for `--mount=type=cache` in apt-get commands, which provides ~10% speedup on cache misses.

### 4. BuildKit Cache Mounts

npm/pip installations use BuildKit cache mounts to persist downloaded packages:

```dockerfile
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g @anthropic-ai/claude-code@${VERSION}
```

This means package downloads are cached within the BuildKit builder, reducing network latency.

### 5. Parallel Execution

With warm cache, `max-parallel: 6` allows efficient parallel builds:

- Runners have 12Gi memory, 4 CPU - can handle 6 concurrent builds
- No resource contention observed
- All agents benefit from pre-warmed cache

## Image Tiering Strategy

The runtime image now supports multiple targets for different use cases:

| Target | Image Tag | Contents | Size | Purpose |
|--------|-----------|----------|------|---------|
| `minimal` | `runtime:latest` | git, Node.js, npm, ripgrep, jq, curl, gh | ~500MB | Agent execution (default) |
| `production` | `runtime:full` | All dev tools, languages, k8s CLIs, scanners | ~2GB | Full development |
| `local` | `runtime:local` | production + locally-built Rust binaries | ~2GB | Local development |

### Building Different Targets

```bash
# Minimal image (default - for agents)
docker build -t ghcr.io/5dlabs/runtime:latest .
docker build --target minimal -t ghcr.io/5dlabs/runtime:minimal .

# Full development environment
docker build --target production -t ghcr.io/5dlabs/runtime:full .

# Local development (builds Rust binaries from source)
docker build --target local -t ghcr.io/5dlabs/runtime:local .
```

### Agent Tool Requirements

Analysis of agent Dockerfiles shows minimal runtime requirements:

| Tool | Claude | Codex | Cursor | Code | Factory | Gemini | OpenCode |
|------|--------|-------|--------|------|---------|--------|----------|
| Node.js/npm | Yes | Yes | - | Yes | - | Yes | Yes |
| git | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| ripgrep | Yes | - | - | Yes | Yes | - | Yes |
| jq | Yes | - | - | Yes | - | - | - |
| gh (GitHub) | Yes | - | - | Yes | - | - | - |
| curl | Yes | Yes | Yes | Yes | Yes | Yes | Yes |

The `minimal` target includes all essential tools for agent execution. Agents that need additional tools can install them at runtime via `sudo apt-get`.

### Layer Optimization Strategy

The Dockerfile uses several techniques to minimize layers:

1. **Consolidated CLI installations**: Multiple CLI tools (helm, argocd, argo, talosctl, yq, stern, docker-compose, k9s, kustomize, cloudflared) are downloaded in a single RUN instruction instead of separate layers.

2. **Combined conditional installs**: CTO binaries (tools, play-monitor, intake) are installed in a single RUN with conditionals.

3. **Merged setup commands**: Rust environment, shell profiles, and scripts are configured in combined RUN instructions.

**Results**: Layer count reduced from ~65 to ~38 for production target (41% reduction).

## Troubleshooting

### Slow Builds

1. **Check cache status**: Look for "CACHED" in build logs. If layers are downloading instead of cached, the registry cache may be corrupted.

   ```bash
   # Trigger with no_cache to reset
   gh workflow run agents-build.yaml --field no_cache=true
   ```

2. **Check warm-cache job**: If warm-cache fails, subsequent agents won't benefit from cache. Cursor is used because it has the simplest Dockerfile.

3. **Check parallelism**: If builds are timing out, reduce `max-parallel` in the workflow.

### Cancelled Builds

Builds were historically cancelled due to `activeDeadlineSeconds` on runners. This has been increased to 7200 (2 hours). If builds still timeout:

1. Check if any single job is stuck (network issues, hung docker daemon)
2. Consider splitting long-running jobs
3. Check runner logs in Kubernetes

### Cache Corruption

If builds are slow despite cache being present:

1. Trigger manual build with `no_cache: true`
2. This rebuilds everything fresh and repopulates cache
3. Subsequent builds should be fast again

## Best Practices

### Adding a New Agent

1. Create Dockerfile in `infra/images/<agent>/`
2. Use standard syntax: `# syntax=docker/dockerfile:1`
3. Use cache mount for package installs:
   ```dockerfile
   RUN --mount=type=cache,target=/root/.npm,sharing=locked \
       npm install -g <package>
   ```
4. Add to matrix in `.github/workflows/agents-build.yaml`
5. Test locally first: `docker build infra/images/<agent>/`

### Modifying Runtime

Changes to `infra/images/runtime/Dockerfile`:
- Will trigger full rebuild (tag includes Dockerfile hash)
- Keep frequently-changing instructions at the end
- Combine apt-get commands to reduce layers
- Test locally before pushing

### Debugging Cache Issues

```bash
# View recent builds
gh run list --workflow agents-build.yaml --limit 10

# Get job timings
gh run view <run-id> --json jobs --jq '.jobs[] | "\(.name): \((.completedAt | fromdateiso8601) - (.startedAt | fromdateiso8601) | . / 60 | floor)m"'

# View build logs
gh run view <run-id> --log | grep -E "(CACHED|downloading|exporting)"
```

## Metrics

Track these metrics to detect performance regressions:

| Metric | Good | Warning | Action |
|--------|------|---------|--------|
| Full pipeline | <15 min | 15-25 min | Check cache |
| Claude build | <5 min | 5-10 min | Check warm-cache |
| Runtime build | <3 min | 3-10 min | Expected (no cache hit) |
| Runtime build | <1 min | - | Great (cache hit) |

## Related Files

- `.github/workflows/agents-build.yaml` - Build workflow
- `infra/images/runtime/Dockerfile` - Base image
- `infra/images/*/Dockerfile` - Agent images
- `infra/gitops/applications/workloads/platform-runners.yaml` - Runner configuration
