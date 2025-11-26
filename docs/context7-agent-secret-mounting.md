# Context7 Secret Mounting for Agents

## Current Status

✅ **Secret Created:**
- Secret `context7-api-key` exists in `agent-platform` namespace
- Contains: `CONTEXT7_API_KEY`
- Synced from `secret-store` namespace via External Secrets

## Problem

Agents need the `CONTEXT7_API_KEY` environment variable to use Context7 as a local MCP server, but the secret is not currently mounted to agent containers.

## Solution Options

### Option 1: Mount via Task Requirements (Per-Task)

Add Context7 secret to task requirements when creating CodeRuns:

```yaml
task_requirements:
  secrets:
    - name: context7-api-key
      keys:
        CONTEXT7_API_KEY: CONTEXT7_API_KEY
```

**Pros:** Flexible, only mounts when needed
**Cons:** Must be specified for every task

### Option 2: Add to Controller Default Secrets (Automatic)

Modify the controller to automatically mount Context7 secret to all agent containers.

**File:** `controller/src/tasks/code/resources.rs`

**Location:** In the `build_job_spec` function, after GitHub App secret mounting

**Add:**
```rust
// Mount Context7 API key for direct MCP connections
env_from.push(json!({
    "secretRef": {
        "name": "context7-api-key"
    }
}));
```

**Pros:** Automatic, works for all agents
**Cons:** Requires controller code change

### Option 3: Add to Helm Values (Configuration)

Add Context7 secret to the controller's Helm values as a default secret.

**File:** `infra/charts/controller/values.yaml`

**Add new section:**
```yaml
# Common secrets mounted to all agent containers
commonSecrets:
  - context7-api-key
```

Then update controller code to read and mount these common secrets.

**Pros:** Configurable via Helm
**Cons:** Requires both Helm and controller changes

### Option 4: Use envFrom in Pod Template (Simplest)

The controller already supports `envFrom` for mounting entire secrets. We just need to ensure `context7-api-key` is added to the `env_from` array.

**Current Code:** Lines 758-761 in `controller/src/tasks/code/resources.rs`

```rust
// Add envFrom if we have secrets to mount
if !env_from.is_empty() {
    container_spec["envFrom"] = json!(env_from);
}
```

**Add before this:**
```rust
// Always mount Context7 API key for direct MCP connections
env_from.push(json!({
    "secretRef": {
        "name": "context7-api-key"
    }
}));
```

## Recommended Solution: Option 4

**Why:** Simplest, requires only one-line code change, works for all agents automatically.

### Implementation Steps:

1. **Update Controller Code**
   
   **File:** `controller/src/tasks/code/resources.rs`
   
   **Line:** ~755 (before the envFrom check)
   
   **Add:**
   ```rust
   // Mount Context7 API key secret for direct MCP connections (all agents)
   env_from.push(json!({
       "secretRef": {
           "name": "context7-api-key"
       }
   }));
   ```

2. **Build and Deploy Controller**
   ```bash
   cd controller
   cargo build --release --bin agent-controller
   # Build and push controller image
   # Update controller deployment
   ```

3. **Verify Secret Mounting**
   ```bash
   # Create a test CodeRun
   # Check the pod has CONTEXT7_API_KEY
   kubectl exec -it <agent-pod> -n agent-platform -- env | grep CONTEXT7_API_KEY
   ```

## Alternative: Quick Test Without Controller Change

For immediate testing, you can add the secret to a specific CodeRun's task requirements:

```yaml
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-context7
spec:
  taskId: 1
  githubApp: "5DLabs-Rex"
  repositoryUrl: "https://github.com/5dlabs/cto-test"
  service: "test"
  model: "claude-sonnet-4-5-20250929"
  cli: "factory"
  taskRequirements:
    secrets:
      - name: context7-api-key
```

This will mount the entire `context7-api-key` secret as environment variables.

## Verification

After mounting the secret, verify in an agent container:

```bash
# Exec into agent pod
kubectl exec -it <agent-pod> -n agent-platform -- bash

# Check environment variable
echo $CONTEXT7_API_KEY
# Should output: ctx7sk-86882734-b6b1-4755-b9ad-2708127f0028

# Test Context7
npx -y @upstash/context7-mcp
# Should connect successfully
```

## Summary

**Current State:**
- ✅ Secret exists in agent-platform namespace
- ❌ Not mounted to agent containers
- ❌ CONTEXT7_API_KEY not available in agent environment

**Required:**
- Add one line to controller code to mount `context7-api-key` secret
- Or use task requirements to mount per-task

**Once Fixed:**
- All agents will have CONTEXT7_API_KEY
- Local Context7 MCP servers will work
- Agents can use Context7 directly (bypassing Tools routing bug)

---

**Recommended Action:** Add secret mounting to controller code (Option 4)
**Quick Test:** Use task requirements for a test CodeRun

