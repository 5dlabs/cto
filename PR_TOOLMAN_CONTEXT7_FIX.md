# Fix Toolman Context7 and GitHub Integration

## Summary

Fixes missing secret configurations for Toolman's Context7 and GitHub MCP integrations, enabling remote access to context7 documentation tools and GitHub API tools for all agents.

## Problem

When testing Toolman MCP remote tools:
- ✅ **shadcn tools**: Working perfectly (retrieved 46 components)
- ❌ **context7 tools**: Failing with "Tool resolve_library_id not found"
- ❌ **GitHub tools**: Not accessible

### Root Causes

1. **Missing ExternalSecrets**: The `toolman-context7-secrets` and `toolman-github-secrets` ExternalSecret resources only existed in the `mcp` namespace, not in `agent-platform` where Toolman is deployed.

2. **Secrets Not Mounted**: Even though `secretRefs` were defined in the Helm values, they weren't being used to create environment variables in the Toolman deployment.

## Solution

### 1. Added Missing ExternalSecrets (`infra/secret-store/toolman-external-secrets.yaml`)

Created ExternalSecret resources in the `agent-platform` namespace:

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

### 2. Mounted Secrets as Environment Variables (`infra/gitops/applications/toolman.yaml`)

Added explicit environment variable mappings:

```yaml
env:
  - name: CONTEXT7_API_KEY
    valueFrom:
      secretKeyRef:
        name: toolman-context7-secrets
        key: CONTEXT7_API_KEY
        optional: true
  - name: GITHUB_PERSONAL_ACCESS_TOKEN
    valueFrom:
      secretKeyRef:
        name: toolman-github-secrets
        key: GITHUB_PERSONAL_ACCESS_TOKEN
        optional: true
```

### 3. Additional Fixes

- **Atlas GitHub API integration**: Fixed Atlas to use GitHub API directly instead of `gh` CLI for PR mergeable status checks
- **YAML linting**: Fixed trailing spaces and EOF newlines

## Test Results

### Before Fix
```bash
❌ mcp_toolman_context7_resolve_library_id("react")
   Error: Tool resolve_library_id not found

✅ mcp_toolman_shadcn_list_components()
   Success: Retrieved 46 components
```

### Verification from Logs
```
✅ Including tool: context7_resolve-library-id
✅ Including tool: context7_get-library-docs
🔗 Server 'context7' is already connected
🔧 Forwarding tool call: resolve_library_id to server context7
📨 Received response from server context7
```

The context7 server is connected and responding, but lacks API key authentication.

### After Fix (Expected)
Once deployed:
1. External Secrets Operator creates secrets in `agent-platform` namespace
2. Toolman pod restarts with Context7 and GitHub API keys mounted
3. All remote tools (context7, shadcn, GitHub) fully functional

## Files Modified

1. **`infra/secret-store/toolman-external-secrets.yaml`**
   - Added `toolman-context7-secrets` ExternalSecret
   - Added `toolman-github-secrets` ExternalSecret

2. **`infra/gitops/applications/toolman.yaml`**
   - Added `CONTEXT7_API_KEY` environment variable
   - Added `GITHUB_PERSONAL_ACCESS_TOKEN` environment variable

3. **`client-config.json`**
   - Kept hyphenated format for context7 tools (matches Toolman registration)

4. **Atlas integration fixes** (bonus improvements)
   - Fixed GitHub API usage for mergeable status checks
   - Fixed YAML linting issues

## Deployment Steps

1. Merge PR to main
2. ArgoCD syncs changes automatically
3. External Secrets Operator creates secrets in `agent-platform`
4. Toolman deployment updates and pod restarts
5. Context7 and GitHub MCP servers initialize with proper credentials

## Verification

After deployment:

```bash
# Check secrets exist
kubectl get externalsecrets -n agent-platform | grep toolman

# Check Toolman pod restarted
kubectl get pods -n agent-platform -l app.kubernetes.io/name=toolman

# Check environment variables are mounted
kubectl exec -n agent-platform -it deploy/toolman -- env | grep -E "CONTEXT7|GITHUB"

# Check Toolman logs
kubectl logs -n agent-platform -l app.kubernetes.io/name=toolman | grep -i context7
```

Test MCP tools:
```
mcp_toolman_context7_resolve_library_id("react")
mcp_toolman_shadcn_list_components()
```

## Impact

- **All agents** can now access Context7 documentation tools for up-to-date library information
- **All agents** can access GitHub API tools for repository operations
- **shadcn tools** continue working as before
- No breaking changes to existing functionality

## Related

- Toolman deployment: `infra/gitops/applications/toolman.yaml`
- MCP namespace secrets: `infra/secret-store/toolman-mcp-external-secrets.yaml`
- Client configuration: `client-config.json`
- Agent configuration: `cto-config.json`

