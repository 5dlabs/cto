# LivePortrait — OVH AI Deploy container

> Archived: LivePortrait was a Gate 1 provider spike and is no longer built,
> deployed, or part of the active avatar provider set. Use LemonSlice or
> EchoMimic for current avatar work.

FastAPI wrapper around [LivePortrait](https://github.com/KlingAIResearch/LivePortrait)
(MIT, KwaiVGI / Kuaishou). Shipped as a single Docker image with the humans-model
weights baked in so OVH AI Deploy can pull and run with zero extra config.

## Endpoints

- `GET  /health` — liveness probe.
- `GET  /`       — basic info.
- `POST /animate` — multipart form:
  - `source` *(required)* — portrait image (jpg/png) or video
  - `driving` *(required)* — driving video (mp4) or `.pkl` template
  - `flag_crop_driving_video` *(bool, default false)*
  - `driving_option` *(expression-friendly | pose-friendly)*
  - `animation_region` *(exp | pose | lip | eyes | all)*
  - `flag_stitching` / `flag_relative_motion` / `flag_pasteback` / `flag_do_crop`
  - `driving_multiplier` *(float, default 1.0)*

Returns `video/mp4`.

## Build

```bash
# Local build (requires Docker + BuildKit). Image is ~12-14GB with weights.
docker build -t ghcr.io/5dlabs/liveportrait:v1 avatar/liveportrait

# Multi-arch is unnecessary — OVH AI Deploy is x86_64 only.
DOCKER_BUILDKIT=1 docker build \
  --platform linux/amd64 \
  -t ghcr.io/5dlabs/liveportrait:v1 \
  avatar/liveportrait
```

## Push

GHCR (preferred, existing auth):

```bash
echo "$GITHUB_TOKEN" | docker login ghcr.io -u 5dlabs --password-stdin
docker push ghcr.io/5dlabs/liveportrait:v1
```

If OVH AI Deploy cannot pull from GHCR (auth modes vary), retag for OVH MPR:

```bash
# OVH's Managed Private Registry URL is returned when we create it via API.
# Placeholder:
docker tag ghcr.io/5dlabs/liveportrait:v1 registry.gra.ai.cloud.ovh.net/5dlabs/liveportrait:v1
docker push              registry.gra.ai.cloud.ovh.net/5dlabs/liveportrait:v1
```

## Deploy to OVH AI Deploy

```bash
# Prereqs: AI Deploy must be activated on the project (see avatar/scripts/ovh-api.sh).
./avatar/scripts/ovh-api.sh POST "/cloud/project/$OVH_PROJECT_ID/ai/app" \
  "$(cat <<'JSON'
  {
    "image": "ghcr.io/5dlabs/liveportrait:v1",
    "region": "GRA",
    "resources": { "flavor": "ai1-1-gpu", "cpu": 0, "gpu": 1 },
    "scalingStrategy": { "fixed": { "replicas": 1 } },
    "defaultHttpPort": 8000,
    "probe": { "path": "/health", "port": 8000 },
    "unsecureHttp": false,
    "name": "liveportrait-gate1"
  }
JSON
  )"
```

(Exact schema is validated against OVH's `/ai/app` spec at launch time.)

## Gate 1 validation

```bash
APP_URL="https://<app>.app.gra.ai.cloud.ovh.net"
curl -sS "$APP_URL/health"

# Drive morgan.jpg with a sample driving video. LivePortrait is video-driven, not
# audio-driven — EchoMimic V3 is the audio-driven Gate 2 fallback.
curl -sS -X POST "$APP_URL/animate" \
  -F "source=@avatar/morgan.jpg" \
  -F "driving=@avatar/liveportrait/samples/driving.mp4" \
  -F "flag_crop_driving_video=true" \
  -o out.mp4
```

## Local smoke test

```bash
# Needs an NVIDIA GPU with >=16GB VRAM on the host.
docker run --rm --gpus all -p 8000:8000 ghcr.io/5dlabs/liveportrait:v1
curl http://localhost:8000/health
```

## Notes

- **Base image**: `nvidia/cuda:11.8.0-cudnn8-runtime-ubuntu22.04`. V100S is Volta;
  CUDA 11.8 + torch 2.3.0 is the validated combo. Do not upgrade to CUDA 12.x —
  V100 works but requires newer driver that OVH's V100S pool may not carry.
- **Image size**: ~12-14GB (torch + cudnn + weights). Cold pull on AI Deploy is
  the dominant cost of first request.
- **Concurrency**: serialized via an asyncio lock — single CUDA context per process.
- **Weights**: humans-only. Animals-mode (xpose.pth, X-Pose deps) is excluded to
  keep the image lean.
