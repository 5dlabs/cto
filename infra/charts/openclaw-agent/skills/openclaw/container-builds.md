# Container Builds (Kaniko)

There is **no Docker daemon** in this environment. You are inside a Kubernetes pod.

Container images are built using the **kaniko sidecar** running alongside your agent container. Both containers share the `/workspace` volume.

## How to Build

1. Write your `Dockerfile` somewhere under `/workspace` (e.g. `/workspace/repos/cto/Dockerfile`)
2. Execute the build via `kubectl exec` into the kaniko sidecar:

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/<image>:<tag> \
  --cache=true \
  --cache-repo=ghcr.io/5dlabs/kaniko-cache
```

## Key Flags

| Flag | Purpose |
|------|---------|
| `--context` | Build context directory (absolute path under `/workspace`) |
| `--dockerfile` | Path to Dockerfile (absolute path) |
| `--destination` | Full image reference to push (e.g. `ghcr.io/5dlabs/myapp:latest`) |
| `--cache=true` | Enable layer caching |
| `--cache-repo` | Registry path for cached layers |
| `--no-push` | Build only, don't push (for testing) |
| `--tar-path=/workspace/image.tar` | Save image as tarball instead of pushing |

## Registry Credentials

The kaniko sidecar has GHCR credentials pre-mounted at `/kaniko/.docker/config.json` (from the `ghcr-secret`). Pushes to `ghcr.io/5dlabs/*` work without additional setup.

## Important

- Do NOT try to run `docker build`, `docker push`, or `docker compose` — they will not work
- The kaniko executor runs **once per invocation** and exits — each build is a separate `kubectl exec`
- Build context must be under `/workspace` (the shared volume)
- Multi-stage builds work normally
