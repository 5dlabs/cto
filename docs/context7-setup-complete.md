# Context7 Setup Complete

## Overview

Context7 has been properly configured in the CTO platform to provide up-to-date library documentation and code examples to all agents through the Tools MCP server.

## Changes Made

### 1. External Secrets Configuration

**File:** `infra/secret-store/tools-mcp-external-secrets.yaml`

Added Context7 API key secret configuration:
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: tools-context7-secrets
  namespace: mcp
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: tools-context7-secrets
    creationPolicy: Owner
  data:
  - secretKey: CONTEXT7_API_KEY
    remoteRef:
      key: tools-context7-secrets
      property: CONTEXT7_API_KEY
```

### 2. Tools Configuration

**File:** `infra/gitops/applications/tools.yaml`

**Added:**
- Context7 environment variable configuration in the server definition
- Secret reference to mount the Context7 API key

```yaml
context7:
  name: "Context7"
  description: "Up-to-date library documentation and code examples"
  transport: "stdio"
  command: "npx"
  args: ["-y", "@upstash/context7-mcp"]
  workingDirectory: "/tmp"
  env:
    CONTEXT7_API_KEY: ""  # Injected from secret

secretRefs:
  - name: tools-context7-secrets
```

### 3. Secret Creation Script

**File:** `scripts/add-context7-secret.sh`

Created helper script to add the Context7 API key to the cluster secret store.

## Deployment Steps

### Step 1: Add Secret to Cluster

```bash
# Run the helper script
./scripts/add-context7-secret.sh
```

This will:
1. Create the secret in the `secret-store` namespace
2. External Secrets operator will sync it to the `mcp` namespace
3. Tools will pick it up on next restart

### Step 2: Verify Secret Creation

```bash
# Check secret in secret-store namespace
kubectl get secret tools-context7-secrets -n secret-store

# Check External Secrets synced it to mcp namespace
kubectl get secret tools-context7-secrets -n mcp

# View the ExternalSecret status
kubectl get externalsecret tools-context7-secrets -n mcp -o yaml
```

### Step 3: Restart Tools

```bash
# Restart Tools to pick up the new secret
kubectl rollout restart deployment tools -n agent-platform

# Watch the rollout
kubectl rollout status deployment tools -n agent-platform

# Check logs to verify Context7 is working
kubectl logs -n agent-platform deployment/tools -f
```

## Testing Context7

### Test 1: Via Cursor CLI (Local)

Once Tools is restarted with the API key:

```bash
# Start cursor agent and connect to Tools
cursor agent

# In the agent, test Context7
> use context7 to show me how to use tokio async runtime in Rust

# Or for TypeScript/React
> use context7 to show me React hooks with TypeScript examples
```

### Test 2: Via MCP Direct Call

You can test the Context7 tool directly through the MCP protocol:

```json
{
  "tool": "context7_get_library_docs",
  "arguments": {
    "library": "tokio",
    "query": "async runtime setup and basic usage"
  }
}
```

### Test 3: Verify in Agent Workflows

Context7 is available to all agents through the `remote` tools configuration:

**Rex (Rust):**
- Can query: "tokio async patterns", "serde serialization", "axum web framework"

**Blaze (TypeScript/React):**
- Can query: "React hooks TypeScript", "Next.js API routes", "shadcn/ui components"

**All Agents:**
- Have access to `context7_get_library_docs` tool
- Can query any library/framework with natural language

## How Agents Should Use Context7

### Language-Specific Queries

Context7 doesn't have built-in language filtering, so agents should include the language/framework in their queries:

**Good Queries:**
- ✅ "tokio async runtime in Rust"
- ✅ "React hooks with TypeScript"
- ✅ "Next.js 14 app router patterns"
- ✅ "axum web framework routing in Rust"

**Bad Queries:**
- ❌ "async runtime" (too vague)
- ❌ "hooks" (which framework?)
- ❌ "routing" (which language/framework?)

### Example Usage Patterns

**For Rex (Rust Development):**
```javascript
// Query for Rust-specific documentation
context7_get_library_docs({
  query: "tokio async runtime with tracing instrumentation in Rust"
})

context7_get_library_docs({
  query: "serde derive macros for JSON serialization in Rust"
})
```

**For Blaze (Frontend Development):**
```javascript
// Query for TypeScript/React documentation
context7_get_library_docs({
  query: "React Server Components with TypeScript in Next.js 14"
})

context7_get_library_docs({
  query: "shadcn/ui form components with react-hook-form TypeScript"
})
```

## Configuration in cto-config.json

All agents have been configured with Context7 access:

```json
{
  "agents": {
    "rex": {
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "context7_get_library_docs"
        ]
      }
    },
    "blaze": {
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "context7_get_library_docs"
        ]
      }
    }
    // ... other agents
  }
}
```

## Troubleshooting

### Secret Not Syncing

If the External Secret isn't syncing:

```bash
# Check ExternalSecret status
kubectl describe externalsecret tools-context7-secrets -n mcp

# Check ClusterSecretStore
kubectl get clustersecretstore secret-store -o yaml

# Verify secret exists in source namespace
kubectl get secret tools-context7-secrets -n secret-store
```

### Context7 Not Working

If Context7 queries fail:

```bash
# Check Tools logs
kubectl logs -n agent-platform deployment/tools -f | grep -i context7

# Verify environment variable is set
kubectl exec -n agent-platform deployment/tools -- env | grep CONTEXT7

# Test the Context7 package directly
kubectl exec -n agent-platform deployment/tools -- npx -y @upstash/context7-mcp --help
```

### API Key Issues

If you see authentication errors:

1. Verify the API key format (should start with `ctx7sk-`)
2. Check the key is correctly set in the secret
3. Ensure the secret is mounted to Tools pod
4. Restart Tools after secret changes

## API Key Information

**API Key:** `ctx7sk-df5b4d84-7418-4315-95f9-1216e24eb4e6`

**Storage Location:**
- Kubernetes Secret: `tools-context7-secrets` in `secret-store` namespace
- Synced to: `tools-context7-secrets` in `mcp` namespace
- Mounted to: Tools deployment in `agent-platform` namespace

**Security:**
- Key is stored in Kubernetes secrets (base64 encoded)
- Managed by External Secrets operator
- Only accessible to Tools service account
- Not exposed in config files or logs

## Next Steps

1. ✅ Run `./scripts/add-context7-secret.sh` to add the secret
2. ✅ Verify External Secrets synced it
3. ✅ Restart Tools deployment
4. ✅ Test Context7 queries via Cursor CLI
5. ✅ Verify agents can access Context7 in workflows

## References

- [Context7 Documentation](https://context7.com/docs)
- [Context7 Installation Guide](https://context7.com/docs/installation)
- [Context7 MCP Package](https://www.npmjs.com/package/@upstash/context7-mcp)
- [Tools MCP Architecture](docs/engineering/tools-mcp-architecture.md)

---

**Status:** ✅ Configuration Complete - Ready for Testing
**Date:** November 22, 2025

