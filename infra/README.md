# Infrastructure Directory Structure

This directory contains all infrastructure configurations for the platform, now enhanced with GitOps capabilities using Argo CD and Argo Workflows.



## 🚀 Quick Start

### Complete GitOps Migration




```bash
./scripts/migrate-to-gitops.sh








```

### Manual Installation




```bash
# Install Argo CD
./scripts/install-argocd.sh

# Install Argo Workflows
./scripts/install-argo-workflows.sh

# Deploy GitOps applications
kubectl apply -f gitops/app-of-apps.yaml








```

## Directory Organization



### `charts/`
Helm charts for platform components:



#### `orchestrator/`
The main orchestrator service Helm chart:


- ServiceAccount and RBAC configuration


- ConfigMaps for configuration


- Deployment and Service definitions


- Ingress configuration


- Automatic per-service workspace PVCs

**GitOps Usage:**




```yaml
# Managed by infra/gitops/applications/orchestrator.yaml
source:
  path: infra/charts/orchestrator
syncPolicy:
  automated:
    prune: true
    selfHeal: true








```



#### `argocd/`
Argo CD installation and configuration:


- Installation values and configuration


- Repository access secrets


- RBAC and project definitions

**Installation:**




```bash
helm install argocd argo/argo-cd \


  --namespace argocd \


  --create-namespace \


  --values charts/argocd/install-argocd.yaml








```



#### `argo-workflows/`
Argo Workflows installation and configuration:


- Installation values and configuration


- RBAC for workflow execution


- Artifact repository configuration

**Installation:**




```bash
helm install argo-workflows argo/argo-workflows \


  --namespace argo \


  --create-namespace \


  --values charts/argo-workflows/install-argo-workflows.yaml








```



### `gitops/`
GitOps application definitions for Argo CD:

#### `applications/`


- `controller.yaml` - Controller service GitOps application


- `argo-workflows.yaml` - Argo Workflows GitOps application


- `monitoring-stack.yaml` - Monitoring infrastructure



#### `projects/`


- `platform-project.yaml` - Argo CD project for platform infrastructure



#### `app-of-apps.yaml`
Root application that manages all other applications (App of Apps pattern)

**Usage:**




```bash
kubectl apply -f gitops/projects/platform-project.yaml
kubectl apply -f gitops/app-of-apps.yaml








```



### `workflow-templates/`
Argo Workflow templates for task execution:

#### `coderun-template.yaml`
Workflow template for CodeRun tasks:


- Multi-step workflow for code implementation


- Git repository management


- Claude Code agent execution


- Result publishing and cleanup

#### `docsrun-template.yaml`
Workflow template for DocsRun tasks:


- Documentation generation workflow


- Repository cloning and setup


- Claude Docs agent execution


- Documentation publishing



#### `workflow-rbac.yaml`
RBAC configuration for workflow execution

**Installation:**




```bash
kubectl apply -f workflow-templates/








```



### `telemetry/`
OpenTelemetry and monitoring configurations:


- `otel-collector/` - OpenTelemetry collector Helm chart


- `telemetry-dashboards/` - Grafana dashboard definitions

**GitOps Usage:**
Managed by `gitops/applications/monitoring-stack.yaml`

### `cluster-config/`
Cluster-specific configurations (not managed by Helm or GitOps):


- `local-path-config-patch.yaml` - Local path provisioner configuration


- `talos-local-path-volume.yaml` - Talos-specific volume configuration


- `otel-prometheus-service.yaml` - Prometheus metrics service (DEPRECATED)

**Note:** These are typically one-time configurations or cluster-specific settings.



### `arc/`
Actions Runner Controller (ARC) setup for GitHub Actions:


- `arc-org-runners.yaml` - GitHub Actions self-hosted runners


- `setup-org-runners.md` - Detailed setup guide



### `scripts/`
Installation and management scripts:



#### GitOps Scripts


- `migrate-to-gitops.sh` - Complete migration to GitOps


- `install-argocd.sh` - Install and configure Argo CD


- `install-argo-workflows.sh` - Install and configure Argo Workflows


- `install-workflow-templates.sh` - Deploy workflow templates


- `setup-argocd-access.sh` - Configure CI/CD access to Argo CD

#### Traditional Scripts


- `setup-agent-secrets.sh` - Setup GitHub agent secrets


- `setup-all-agents.sh` - Setup multiple agents


- Various monitoring and telemetry scripts



### `test-resources/`
Test manifests and simulators (not for production use):


- `simulators/` - Claude Code telemetry simulators


- `test-pods/` - Test pods for validation


- Various test jobs and configurations



## 🌊 GitOps Workflow

### Deployment Process


1. **Code Changes** → Push to repository


2. **CI/CD** → Build container images


3. **GitOps Update** → Update application manifests


4. **Argo CD** → Detect changes and sync


5. **Kubernetes** → Apply changes automatically

### Task Execution Process


1. **Task Submission** → Via API or webhook


2. **Workflow Creation** → Orchestrator creates Argo Workflow


3. **Workflow Execution** → Argo Workflows manages execution


4. **Agent Execution** → Claude agents run in workflow steps


5. **Result Publishing** → Automated result handling

## 📊 Monitoring & Observability



### Argo CD
- **UI**: http://localhost:30080 (NodePort)
- **Port Forward**: `kubectl port-forward svc/argocd-server -n argocd 8080:443`
- **CLI**: `argocd app list`



### Argo Workflows
- **UI**: http://localhost:30081 (NodePort)
- **Port Forward**: `kubectl port-forward svc/argo-workflows-server -n argo 2746:2746`
- **CLI**: `kubectl get workflows -n argo`

### Application Health


- Automatic health monitoring


- Sync status tracking


- Drift detection and correction



## 🔐 Security



### GitOps Security


- Git-based access control


- Repository credentials management


- RBAC for applications and workflows



### Workflow Security


- Service account isolation


- Resource limits and quotas


- Secret management for agent credentials

## 🚨 Troubleshooting

### Application Issues




```bash
# Check application status
kubectl get applications -n argocd

# Manual sync
kubectl patch application orchestrator -n argocd --type merge -p '{"operation":{"sync":{"revision":"HEAD"}}}'



# View logs
kubectl logs -n argocd deployment/argocd-application-controller








```



### Workflow Issues




```bash


# Check workflows
kubectl get workflows -n argo



# View workflow details
kubectl describe workflow WORKFLOW_NAME -n argo



# Check workflow logs
kubectl logs -n argo -l workflows.argoproj.io/workflow=WORKFLOW_NAME








```



### Repository Access




```bash


# Check repository secrets
kubectl get secrets -n argocd | grep repo

# Update credentials
kubectl apply -f charts/argocd/secrets.yaml








```

## 📚 Migration Guide

Migration instructions and troubleshooting are covered in the sections above. For detailed GitOps migration steps, see the deployment process section.



## 🔄 Legacy Support

The traditional Helm-based deployment is still supported for environments not ready for GitOps:





```bash
# Traditional deployment
helm install orchestrator ./charts/orchestrator \
  --namespace orchestrator \
  --create-namespace \
  --set secrets.anthropicApiKey="your-anthropic-key" \
  --set secrets.openaiApiKey="your-openai-key"








```

However, GitOps is the recommended approach for new deployments.
