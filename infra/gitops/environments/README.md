# Multi-Environment Configuration

This directory contains environment-specific configuration values for the CTO platform.

## Environment Overview

| Environment | Namespace | Branch | Purpose |
|------------|-----------|--------|---------|
| **dev** | `cto-dev` | feature branches | Local development, feature testing |
| **staging** | `cto-staging` | `develop` | Pre-production integration testing |
| **prod** | `cto` | `main` | Production workloads |

## Directory Structure

```
environments/
├── dev/           # Development environment values
│   ├── values.yaml          # Shared dev overrides
│   ├── controller.yaml      # Controller-specific dev config
│   └── ...
├── staging/       # Staging environment values
│   ├── values.yaml          # Shared staging overrides
│   ├── controller.yaml      # Controller-specific staging config
│   └── ...
└── prod/          # Production environment values
    ├── values.yaml          # Shared prod overrides (reference only)
    └── ...
```

## Branch Strategy

- **Feature branches** → Deploy to `cto-dev` namespace (manual or on-demand)
- **develop branch** → Auto-deploy to `cto-staging` namespace
- **main branch** → Auto-deploy to `cto` namespace (production)

## Promotion Flow

```
feature/* → develop → main
    │          │        │
    ↓          ↓        ↓
 cto-dev  cto-staging  cto (prod)
```

## Image Tags

| Environment | Image Tag Strategy |
|------------|-------------------|
| dev | `latest` or feature branch SHA |
| staging | `develop` or specific version |
| prod | Semantic versioned tags (`v*.*.*`) via ArgoCD Image Updater |

## Resource Scaling

| Component | Dev | Staging | Prod |
|-----------|-----|---------|------|
| Controller CPU | 200m-500m | 500m-1000m | 500m-1000m |
| Controller Memory | 512Mi-1Gi | 1Gi-4Gi | 1Gi-8Gi |
| Tools CPU | 100m-200m | 200m-500m | 200m-1000m |
| Tools Memory | 256Mi-512Mi | 512Mi-1Gi | 512Mi-2Gi |
| Replicas | 1 | 1 | 1 (HPA optional) |

