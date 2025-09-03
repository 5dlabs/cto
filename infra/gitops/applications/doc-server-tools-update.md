# Doc Server Tools Configuration Update

## Overview
This update moves the tools.json configuration from being embedded in the binary to being managed via a Kubernetes ConfigMap. This allows runtime configuration changes without rebuilding the container.

## Changes Made

### 1. Helm Chart Updates
- **Created ConfigMap template** (`docs/charts/agent-docs/templates/configmap.yaml`): Generates a ConfigMap containing the tools.json configuration
- **Updated Deployment** (`docs/charts/agent-docs/templates/deployment.yaml`): Mounts the ConfigMap as `/app/tools.json` in the container
- **Updated values.yaml** (`docs/charts/agent-docs/values.yaml`): Added `toolsConfig` field with default tools configuration

### 2. ArgoCD Application Update
The `doc-server.yaml` ArgoCD application needs to be updated with the tools configuration. The tools are defined in `doc-server-tools.json`.

## How to Apply

### Step 1: Update ArgoCD Application
Add the following to the `helm.values` section in `doc-server.yaml`:

```yaml
        toolsConfig: |
          <contents of doc-server-tools.json>
```

### Step 2: Commit and Push Changes to Charts Repository
The Helm chart changes need to be committed to the https://github.com/5dlabs/charts repository:

```bash
cd docs/charts
git add agent-docs/templates/configmap.yaml
git add agent-docs/templates/deployment.yaml
git add agent-docs/values.yaml
git commit -m "feat: Add ConfigMap support for tools.json configuration"
git push origin main
```

### Step 3: Apply ArgoCD Application
```bash
kubectl apply -f docs/cto/infra/gitops/applications/doc-server.yaml
```

### Step 4: Verify Configuration
Once deployed, verify the configuration is loaded correctly:

```bash
# Check if ConfigMap was created
kubectl get configmap -n mcp | grep agent-docs

# Check if tools.json is mounted
kubectl exec -n mcp deployment/agent-docs-server -- ls -la /app/tools.json

# Query the server to verify tools
curl -X POST http://doc-server-agent-docs-server.mcp.svc.cluster.local:80/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | jq .
```

## Configuration Loading Priority
The server will check for configuration in this order:
1. Environment variable `TOOLS_CONFIG` (if set)
2. File at `/app/tools.json` (from ConfigMap)
3. Embedded configuration (fallback)

## Benefits
- **Runtime Configuration**: Tools can be added/removed/modified without rebuilding the container
- **GitOps Management**: Configuration is managed through ArgoCD and stored in Git
- **Easier Testing**: Different environments can have different tool configurations
- **Faster Deployments**: No need to rebuild and push new images for configuration changes

## Notes
- The Jupiter Protocol tool (`jupiter_query`) is now correctly included in the configuration
- All other tools from the original configuration are preserved
- The configuration is validated at startup to ensure it's valid JSON and contains required fields