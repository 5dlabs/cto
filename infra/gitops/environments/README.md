# Environment Configuration

This directory contains environment-specific configuration values for the CTO platform.

## Environment Overview

| Environment | Namespace | Branch | Purpose |
|------------|-----------|--------|---------|
| **prod** | `cto` | `main` | Production workloads |

## Directory Structure

```
environments/
└── prod/          # Production environment values
    └── values.yaml
```

## Deployment Strategy

- **main branch** → Auto-deploy to `cto` namespace (production)

## Image Tags

| Environment | Image Tag Strategy |
|------------|-------------------|
| prod | Semantic versioned tags (`v*.*.*`) via ArgoCD Image Updater |

## Resource Configuration

| Component | Prod |
|-----------|------|
| Controller CPU | 500m-1000m |
| Controller Memory | 1Gi-8Gi |
| Tools CPU | 200m-1000m |
| Tools Memory | 512Mi-2Gi |
| Replicas | 1 (HPA optional) |
