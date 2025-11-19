# GitOps Configuration

This directory contains the Argo CD configuration for managing platform infrastructure via GitOps.



## Structure









```
gitops/
├── app-of-apps.yaml          # Root application managing all others
├── projects/
│   └── platform-project.yaml # Platform project configuration
└── applications/
    ├── controller.yaml       # Controller service
    ├── argo-workflows.yaml   # Argo Workflows
    └── monitoring-stack.yaml # Monitoring infrastructure








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
- **Path**: `infra/charts/controller`
- **Namespace**: `agent-platform`
- **Auto-sync**: Enabled
- **Self-heal**: Enabled



### Argo Workflows
- **Path**: `infra/charts/argo-workflows`
- **Namespace**: `argo`
- **Auto-sync**: Enabled
- **Self-heal**: Enabled

### Monitoring Stack
- **Path**: `infra/telemetry`
- **Namespace**: `monitoring`
- **Auto-sync**: Enabled
- **Self-heal**: Enabled



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

## Monitoring

Applications are automatically monitored by Argo CD:


- Sync status


- Health status


- Resource drift detection


- Automatic remediation

## Troubleshooting

### Check Application Status




```bash
kubectl get applications -n argocd
kubectl describe application controller -n argocd








```

### Manual Sync




```bash


# Via CLI
argocd app sync controller



# Via kubectl
kubectl patch application controller -n argocd --type merge -p '{"operation":{"sync":{"revision":"HEAD"}}}'








```



### View Logs




```bash
kubectl logs -n argocd deployment/argocd-application-controller
kubectl logs -n argocd deployment/argocd-server








```# Security scan trigger - Wed Nov 19 07:12:58 PST 2025
