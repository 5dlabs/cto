# Tweakcn Docker Image

Self-hosted visual theme editor for shadcn/ui components.

## Overview

This image builds [tweakcn](https://github.com/jnsahaj/tweakcn), an open-source visual theme editor
for shadcn/ui components. It provides:

- Visual no-code theme editing
- Pre-built beautiful theme presets
- Real-time component preview
- Export generated CSS/Tailwind config

## Usage

### Local Development

```bash
docker build -t tweakcn .
docker run -p 3000:3000 tweakcn
```

### Access via Kubernetes

The service is deployed to the `cto` namespace and accessible via:

```
http://tweakcn.cto.svc.cluster.local:3000
```

Via Kilo/WireGuard VPN, access directly at the service URL.

## Notes

This is a stateless deployment for basic theme editing. Features requiring
authentication (saving themes, AI generation) are not configured. For those
features, you'd need to configure:

- `DATABASE_URL` - PostgreSQL connection
- `BETTER_AUTH_SECRET` - Auth encryption key
- OAuth credentials for GitHub/Google login
- AI API keys for theme generation

## Source

- **Upstream**: https://github.com/jnsahaj/tweakcn
- **License**: Apache-2.0
