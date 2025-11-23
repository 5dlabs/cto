# Toolman Context7 and GitHub Secrets Fix

## Summary

Fixed missing ExternalSecret configurations for Toolman's context7 and GitHub integrations in the `agent-platform` namespace, and updated client configuration to use remote Toolman tools with correct naming conventions.

## Problem

When testing Toolman MCP integration:
- ✅ **shadcn tools**: Working perfectly (retrieved 46 components)
- ❌ **context7 tools**: Failing with "Tool resolve_library_id not found" error

### Root Cause

The Toolman deployment in `agent-platform` namespace referenced two secrets:
- `toolman-context7-secrets`
- `toolman-github-secrets`

However, these ExternalSecret resources only existed in the `mcp` namespace, not in `agent-platform` where Toolman is deployed. This caused the Context7 API key to be empty, preventing the context7 MCP server from initializing properly.

## Solution

### 1. Added Missing ExternalSecrets

Added two new ExternalSecret definitions to `infra/secret-store/toolman-external-secrets.yaml`:

```yaml
---
# ToolMan Context7 External Secret
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: toolman-context7-secrets
  namespace: agent-platform
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: toolman-context7-secrets
    creationPolicy: Owner
  data:
  - secretKey: CONTEXT7_API_KEY
    remoteRef:
      key: toolman-context7-secrets
      property: CONTEXT7_API_KEY

---
# ToolMan GitHub External Secret
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: toolman-github-secrets
  namespace: agent-platform
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: toolman-github-secrets
    creationPolicy: Owner
  data:
  - secretKey: GITHUB_PERSONAL_ACCESS_TOKEN
    remoteRef:
      key: toolman-github-secrets
      property: GITHUB_PERSONAL_ACCESS_TOKEN
```

### 2. Updated Client Configuration

Updated `client-config.json` to properly configure remote Toolman access:

**Fixed context7 tool naming:**
- Changed from `context7_resolve-library-id` (hyphenated) 
- To `context7_resolve_library_id` (underscored)
- Changed from `context7_get-library-docs` (hyphenated)
- To `context7_get_library_docs` (underscored)

**Added shadcn remote tools:**
- `shadcn_list_components`
- `shadcn_get_component`
- `shadcn_get_component_metadata`
- `shadcn_get_component_demo`
- `shadcn_list_blocks`
- `shadcn_get_block`
- `shadcn_get_directory_structure`

## Test Results

### Before Fix
```
❌ context7_resolve_library_id("react")
   Error: Tool resolve_library_id not found

✅ shadcn_list_components()
   Success: Retrieved 46 components
```

### After Fix (Expected)
Once merged to main and synced by ArgoCD:
1. ExternalSecrets will create the required secrets in `agent-platform` namespace
2. Toolman pod will restart and pick up the Context7 API key
3. Both context7 and shadcn tools will work via remote Toolman

## Files Modified

1. **`client-config.json`**
   - Fixed context7 tool naming (hyphenated → underscored)
   - Added all shadcn remote tools
   - Configured for remote Toolman access

2. **`infra/secret-store/toolman-external-secrets.yaml`**
   - Added `toolman-context7-secrets` ExternalSecret
   - Added `toolman-github-secrets` ExternalSecret

## Deployment Steps

1. Merge PR to main branch
2. ArgoCD will sync the changes automatically
3. External Secrets Operator will create the secrets in `agent-platform` namespace
4. Toolman deployment will restart and mount the new secrets
5. Context7 MCP server will initialize with proper API key
6. All remote tools (context7, shadcn, GitHub) will be functional

## Verification

After deployment, verify with:

```bash
# Check secrets exist
kubectl get externalsecrets -n agent-platform | grep toolman

# Check Toolman pod has restarted
kubectl get pods -n agent-platform -l app.kubernetes.io/name=toolman

# Check Toolman logs for context7 initialization
kubectl logs -n agent-platform -l app.kubernetes.io/name=toolman | grep -i context7
```

Test MCP tools:
```
mcp_toolman_context7_resolve_library_id("react")
mcp_toolman_shadcn_list_components()
```

## Related Documentation

- Toolman deployment: `infra/gitops/applications/toolman.yaml`
- MCP namespace secrets: `infra/secret-store/toolman-mcp-external-secrets.yaml`
- Client configuration: `client-config.json`
- CTO agent configuration: `cto-config.json`
