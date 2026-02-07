# Container Builds (Kaniko)

Adapted from [Docker Essentials](https://clawhub.ai/Arnarsson/docker-essentials) for the Kubernetes/kaniko environment.

There is **no Docker daemon** in this environment. You are inside a Kubernetes pod. All `docker` commands must be translated to their kaniko equivalents below.

Container images are built using the **kaniko sidecar** running alongside your agent container. Both containers share the `/workspace` volume.

## Quick Reference: Docker → Kaniko Translation

| Docker command | Kaniko equivalent |
|---|---|
| `docker build -t img:tag .` | `kubectl exec $(hostname) -c kaniko -- /kaniko/executor --context=. --destination=img:tag` |
| `docker build -f Dockerfile.dev .` | Add `--dockerfile=/workspace/path/Dockerfile.dev` |
| `docker build --no-cache .` | Add `--cache=false` |
| `docker build --build-arg VER=1 .` | Add `--build-arg VER=1` |
| `docker push img:tag` | Automatic — kaniko pushes on successful build |
| `docker build` (no push) | Add `--no-push` |

Commands that **do not apply** (no daemon): `docker run`, `docker ps`, `docker stop`, `docker rm`, `docker exec`, `docker logs`, `docker-compose`, `docker network`, `docker volume`, `docker system`.

## Building Images

### Basic build and push

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/myapp:1.0
```

### Build with caching (recommended)

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/myapp:1.0 \
  --cache=true \
  --cache-repo=ghcr.io/5dlabs/kaniko-cache
```

### Build with custom Dockerfile

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile.dev \
  --destination=ghcr.io/5dlabs/myapp:dev
```

### Build with build args

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/myapp:1.0 \
  --build-arg VERSION=1.0 \
  --build-arg NODE_ENV=production
```

### Build without pushing (test only)

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --no-push
```

### Build and save as tarball

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --no-push \
  --tar-path=/workspace/image.tar
```

### Multiple tags

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/myapp:1.0 \
  --destination=ghcr.io/5dlabs/myapp:latest
```

## Executor Flags Reference

| Flag | Purpose |
|------|---------|
| `--context` | Build context directory (absolute path under `/workspace`) |
| `--dockerfile` | Path to Dockerfile (absolute, defaults to `context/Dockerfile`) |
| `--destination` | Image reference to push (repeatable for multiple tags) |
| `--cache=true` | Enable layer caching |
| `--cache-repo` | Registry path for cached layers |
| `--no-push` | Build only, don't push |
| `--tar-path` | Save image as tarball |
| `--build-arg` | Set build-time variable (repeatable) |
| `--target` | Set target build stage for multi-stage builds |
| `--skip-tls-verify` | Skip TLS verification (insecure registries) |
| `--verbosity` | Log level: `panic`, `fatal`, `error`, `warn`, `info`, `debug`, `trace` |
| `--snapshot-mode=redo` | Faster builds, slightly larger images |
| `--use-new-run` | Experimental faster RUN execution |

## Writing Dockerfiles

Dockerfile syntax is identical — kaniko supports the full Dockerfile specification.

### Multi-stage build (recommended for small images)

```dockerfile
FROM node:18 AS builder
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
```

### Build with specific target stage

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/myapp:dev \
  --target=builder
```

### Dockerfile tips

- Use `.dockerignore` to exclude files from build context (reduces build time)
- Combine `RUN` commands to reduce layers: `RUN apt-get update && apt-get install -y pkg`
- Put frequently changing steps (COPY source) after rarely changing steps (COPY package.json + RUN install)
- Use multi-stage builds to keep final images small
- Always tag images with versions, not just `latest`
- Pin base image versions for reproducibility: `FROM node:18.19-slim` not `FROM node`

## Registry Credentials

The kaniko sidecar has GHCR credentials pre-mounted at `/kaniko/.docker/config.json` (from `ghcr-secret`). Pushes to `ghcr.io/5dlabs/*` work without additional setup.

## Important

- Do NOT run `docker build`, `docker push`, or `docker compose` — there is no Docker daemon
- Each `kubectl exec` into kaniko runs **one build** and exits — it is not a persistent daemon
- Build context must be under `/workspace` (the shared volume between containers)
- Kaniko supports the full Dockerfile spec including multi-stage, ARG, ONBUILD, etc.
