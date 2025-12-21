# GitOps Configuration

This directory contains the Argo CD configuration for managing platform infrastructure via GitOps.

## Architecture

The CTO platform deploys to the `cto` namespace from the `main` branch with automatic syncing.

| Environment | Namespace | Branch | App of Apps | Auto-Sync |
|-------------|-----------|--------|-------------|-----------|
| **Production** | `cto` | `main` | `app-of-apps.yaml` | ✅ Auto |

## Directory Structure

```
gitops/
├── app-of-apps.yaml              # Production (main branch)
├── projects/
│   └── platform-project.yaml     # ArgoCD project with RBAC
├── applications/
│   ├── cto/                      # Application manifests
│   │   ├── controller.yaml
│   │   ├── tools.yaml
│   │   ├── healer.yaml
│   │   ├── pm.yaml
│   │   └── research.yaml
│   ├── argo-workflows.yaml       # Shared infrastructure
│   └── monitoring-stack.yaml
├── environments/                 # Environment-specific values
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

### 4. Deploy App of Apps

```bash
kubectl apply -f infra/gitops/app-of-apps.yaml
```

## Applications

### Controller

- **Chart**: `infra/charts/cto`
- **Namespace**: `cto`
- **Image Updater**: Enabled

### Tools

- **Chart**: `infra/charts/tools`
- **Namespace**: `cto`
- **Image Updater**: Enabled

### Healer

- **Chart**: `infra/charts/universal-app`
- **Namespace**: `cto`

### PM Server

- **Chart**: `infra/charts/pm`
- **Namespace**: `cto`

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
# All applications
kubectl get applications -n argocd

# Application details
kubectl describe application cto-controller -n argocd
```

### Manual Sync

```bash
# Via CLI
argocd app sync cto-controller

# Via kubectl
kubectl patch application cto-controller -n argocd \
  --type merge -p '{"operation":{"sync":{"revision":"HEAD"}}}'
```

### View Logs

```bash
kubectl logs -n argocd deployment/argocd-application-controller
kubectl logs -n argocd deployment/argocd-server
```

### Check Environment Health

```bash
# List all pods
kubectl get pods -n cto

# Check controller logs
kubectl logs -n cto deployment/cto-controller
```
