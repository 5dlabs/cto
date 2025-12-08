# GitOps Configuration

This directory contains the Argo CD configuration for managing platform infrastructure via GitOps.

## Multi-Environment Architecture

The CTO platform supports three environments with isolated namespaces:

| Environment | Namespace | Branch | App of Apps | Auto-Sync |
|-------------|-----------|--------|-------------|-----------|
| **Development** | `cto-dev` | feature branches | `app-of-apps-dev.yaml` | Manual |
| **Staging** | `cto-staging` | `develop` | `app-of-apps-staging.yaml` | ✅ Auto |
| **Production** | `cto` | `main` | `app-of-apps.yaml` | ✅ Auto |

### Promotion Flow

```
feature/* ──→ develop ──→ main
    │            │          │
    ↓            ↓          ↓
 cto-dev    cto-staging    cto (prod)
```

## Directory Structure

```
gitops/
├── app-of-apps.yaml              # Production (main branch)
├── app-of-apps-staging.yaml      # Staging (develop branch)
├── app-of-apps-dev.yaml          # Development (feature branches)
├── projects/
│   └── platform-project.yaml     # ArgoCD project with RBAC
├── applications/
│   ├── cto/                      # Production application manifests
│   │   ├── controller.yaml
│   │   ├── tools.yaml
│   │   ├── healer.yaml
│   │   ├── pm.yaml
│   │   └── research.yaml
│   ├── dev/                      # Development application manifests
│   │   ├── controller.yaml
│   │   └── tools.yaml
│   ├── staging/                  # Staging application manifests
│   │   ├── controller.yaml
│   │   ├── tools.yaml
│   │   ├── healer.yaml
│   │   └── pm.yaml
│   ├── argo-workflows.yaml       # Shared infrastructure
│   └── monitoring-stack.yaml
├── environments/                 # Environment-specific values
│   ├── dev/
│   ├── staging/
│   └── prod/
└── resources/                    # Additional K8s resources
```

## Getting Started

### 1. Install Argo CD

```bash
./infra/scripts/install-argocd.sh
```

### 2. Configure Repository Access

Update secrets in `infra/charts/argocd/secrets.yaml` with your GitHub credentials.

### 3. Create Platform Project

```bash
kubectl apply -f infra/gitops/projects/platform-project.yaml
```

### 4. Deploy App of Apps (Production)

```bash
kubectl apply -f infra/gitops/app-of-apps.yaml
```

### 5. Deploy Staging Environment (Optional)

```bash
kubectl apply -f infra/gitops/app-of-apps-staging.yaml
```

### 6. Deploy Development Environment (Optional)

```bash
kubectl apply -f infra/gitops/app-of-apps-dev.yaml
```

## Environment Details

### Development (cto-dev)

- **Purpose**: Feature development and testing
- **Branch**: Any feature branch (HEAD)
- **Sync**: Manual - developers control when to deploy
- **Self-heal**: Disabled - allows manual interventions
- **Resources**: Reduced (500m CPU, 1Gi memory)
- **Cleanup**: Fast (2 min success, 15 min failure TTL)

### Staging (cto-staging)

- **Purpose**: Pre-production integration testing
- **Branch**: `develop`
- **Sync**: Automatic with self-heal
- **Image Tag**: `develop` (rebuilt on each push)
- **Resources**: Medium (1000m CPU, 4Gi memory)
- **Cleanup**: Moderate (5 min success, 30 min failure TTL)

### Production (cto)

- **Purpose**: Live workloads
- **Branch**: `main`
- **Sync**: Automatic with self-heal
- **Image Tag**: Semver tags via ArgoCD Image Updater
- **Resources**: Full (1000m CPU, 8Gi memory)
- **Cleanup**: Standard (5 min success, 60 min failure TTL)

## Applications

### Controller

- **Chart**: `infra/charts/controller`
- **Namespaces**: `cto`, `cto-staging`, `cto-dev`
- **Image Updater**: Enabled (production only)

### Tools

- **Chart**: `infra/charts/tools`
- **Namespaces**: `cto`, `cto-staging`, `cto-dev`
- **Image Updater**: Enabled (production only)

### Healer

- **Chart**: `infra/charts/universal-app`
- **Namespaces**: `cto`, `cto-staging`
- **Note**: Staging runs in dry-run mode

### PM Server

- **Chart**: `infra/charts/pm`
- **Namespaces**: `cto`, `cto-staging`
- **Webhooks**: Environment-specific URLs

## Access

### Argo CD UI

- **URL**: http://localhost:30080 (NodePort)
- **Port Forward**: `kubectl port-forward svc/argocd-server -n argocd 8080:443`

### Argo Workflows UI

- **URL**: http://localhost:30081 (NodePort)
- **Port Forward**: `kubectl port-forward svc/argo-workflows-server -n argo 2746:2746`

## Security

- Repository access is configured via secrets
- Project-based RBAC controls application permissions
- Automated sync with prune and self-heal enabled
- Resource whitelists prevent unauthorized deployments

## Troubleshooting

### Check Application Status

```bash
# All environments
kubectl get applications -n argocd

# Specific environment
kubectl get applications -n argocd -l environment=staging

# Application details
kubectl describe application cto-controller-staging -n argocd
```

### Manual Sync

```bash
# Via CLI
argocd app sync cto-controller-staging

# Via kubectl
kubectl patch application cto-controller-staging -n argocd \
  --type merge -p '{"operation":{"sync":{"revision":"HEAD"}}}'
```

### View Logs

```bash
kubectl logs -n argocd deployment/argocd-application-controller
kubectl logs -n argocd deployment/argocd-server
```

### Check Environment Health

```bash
# List all pods in an environment
kubectl get pods -n cto-staging

# Check controller logs
kubectl logs -n cto-staging deployment/cto-controller
```
