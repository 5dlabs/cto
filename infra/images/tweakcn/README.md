# Tweakcn Docker Image

Self-hosted visual theme editor for shadcn/ui components.

## Overview

This image builds [tweakcn](https://github.com/jnsahaj/tweakcn), an open-source visual theme editor
for shadcn/ui components. It provides:

- Visual no-code theme editing
- Pre-built beautiful theme presets
- Real-time component preview
- Theme saving and management
- AI theme generation (requires API keys)

## Usage

### Local Development

```bash
docker build -t tweakcn .
docker run -p 3000:3000 -v tweakcn-data:/data tweakcn
```

### Access via Kubernetes

The service is deployed to the `cto` namespace and accessible via:

```
http://tweakcn.cto.svc.cluster.local:3000
```

Via Kilo/WireGuard VPN, access directly at the service URL.

## Configuration

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `NODE_ENV` | Environment mode | `production` |
| `DATABASE_URL` | SQLite database path | `file:/data/tweakcn.db` |
| `PORT` | Server port | `3000` |

## Persistence

SQLite database stored at `/data/tweakcn.db`. Mount a persistent volume to `/data` to preserve themes.

## Source

- **Upstream**: https://github.com/jnsahaj/tweakcn
- **License**: Apache-2.0

