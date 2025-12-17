# Workflow Setup Guide

This document describes the infrastructure required for Argo Workflows to run intake and play workflows in the CTO namespace.

## Components Managed by Helm Chart

These resources are automatically created when the chart is deployed:

### 1. Service Accounts & RBAC

- **`argo-workflow`** - Service account used by workflow pods
  - File: `templates/controller/workflow-rbac.yaml`
  - Permissions: pods, configmaps, secrets, workflows

- **`cto-pm`** - Service account for the PM service  
  - File: `templates/pm/rbac.yaml`
  - Permissions: configmaps, pods, workflows (cluster-wide)

### 2. ConfigMaps

- **`controller-agents`** - Agent metadata configuration
  - File: `templates/controller/agent-templates.yaml`
  - Contents: JSON with agent definitions
  - **Usage**: Mounted at `/config/agents` in workflow pods (currently unused but required)

- **`controller-agent-templates-intake`** - Intake workflow script
  - File: `templates/controller/agent-templates.yaml`
  - Contents: `intake_intake.sh` - Full unified intake script
  - **Usage**: Mounted at `/agent-templates/intake_intake.sh`

## Manual Setup Required

These resources must be created manually or via ExternalSecrets:

### 1. Secrets

**`agent-platform-secrets`** - API keys for AI providers
```bash
kubectl create secret generic agent-platform-secrets -n cto \
  --from-literal=ANTHROPIC_API_KEY="sk-ant-..." \
  --from-literal=OPENAI_API_KEY="sk-..."
```

**`github-app-5dlabs-morgan`** - GitHub App credentials
```bash
kubectl create secret generic github-app-5dlabs-morgan -n cto \
  --from-literal=app-id="..." \
  --from-literal=private-key="..." \
  --from-literal=client-id="..." \
  --from-literal=client-secret="..."
```

### 2. WorkflowTemplate

The `project-intake` WorkflowTemplate must be deployed to the `cto` namespace.
Currently managed in `infra/gitops/manifests/argo-workflows/`.

Image configuration:
- Production: `ghcr.io/5dlabs/controller:latest`
- Dev (Tilt): `192.168.1.72:30500/controller:tilt-dev`

## Verification

```bash
# Check all required resources exist
kubectl get sa argo-workflow cto-pm -n cto
kubectl get role argo-workflow cto-pm -n cto  
kubectl get rolebinding argo-workflow cto-pm -n cto
kubectl get configmap controller-agents controller-agent-templates-intake -n cto
kubectl get secret agent-platform-secrets github-app-5dlabs-morgan -n cto
kubectl get workflowtemplate project-intake -n cto
```

## Troubleshooting

### "secret not found" errors
Ensure `agent-platform-secrets` exists with keys `ANTHROPIC_API_KEY` and `OPENAI_API_KEY`.

### "configmap not found" errors  
Run `helm upgrade cto ./infra/charts/cto` to create ConfigMaps from templates.

### "workflowtemplate not found"
Deploy the WorkflowTemplate from gitops manifests or copy from `default` namespace:
```bash
kubectl get workflowtemplate project-intake -n default -o yaml | \
  sed 's/namespace: default/namespace: cto/' | \
  kubectl apply -f -
```















