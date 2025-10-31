# CodeRun Empty cliType Root Cause Analysis

## Summary

**Critical Issue:** One CodeRun with empty `cliType` breaks controller reconciliation for ALL CodeRuns.

**Impact:** Workflows stall indefinitely, no new tasks can start, entire platform blocked.

## Root Cause Chain

### 1. Workflow Created Without `security-agent` Parameter

**Trigger:** MCP tool submits workflow without security-agent parameter (old MCP binary or missing config)

**Example:** Workflow `play-workflow-template-bst6g` created at `2025-10-31T16:37:34Z`
```yaml
parameters:
  - implementation-agent: "5DLabs-Rex"
  - quality-agent: "5DLabs-Cleo"
  - testing-agent: "5DLabs-Tess"
  - security-agent: ""  # ❌ MISSING/EMPTY
```

### 2. Workflow Template Passes Empty Values

**File:** `play-workflow-template.yaml` line 457-458

```yaml
- name: github-app
  value: "{{workflow.parameters.security-agent}}"  # → ""
- name: cli-type
  value: "{{workflow.parameters.security-cli}}"    # → ""
- name: model
  value: "{{workflow.parameters.security-model}}"  # → ""
```

**Problem:** No validation - empty strings pass through unchecked.

### 3. CodeRun Created With Invalid Fields

**Result:**
```yaml
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: cto-parallel-test-t5-security-crnvp
spec:
  githubApp: ""           # ❌ Empty - controller can't determine which agent
  cliConfig:
    cliType: ""           # ❌ Empty - deserialize fails!
    model: ""             # ❌ Empty - no model specified
```

### 4. Controller List Operation Fails

**File:** `controller/src/crds/coderun.rs` line 48-51

```rust
pub struct CLIConfig {
    #[serde(rename = "cliType")]
    pub cli_type: CLIType,  // NOT Optional - requires valid variant
```

**File:** `controller/src/cli/types.rs` line 67-88

```rust
impl<'de> Deserialize<'de> for CLIType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        const VARIANTS: &[&str] = &["claude", "codex", ...];
        
        CLIType::from_str_ci(&value)
            .ok_or_else(|| serde::de::Error::unknown_variant(&value, VARIANTS))
            //  ↑ Empty string causes: unknown variant '', expected one of ...
    }
}
```

**Controller Error:**
```
Error("unknown variant ``, expected one of `claude`, `codex`, ...", line: 1, column: 29477)
code_reconciliation_result: CodeRun reconciliation error
  error=QueueError(InitialListFailed(SerdeError(...)))
```

**Critical Impact:** 
- Controller calls `coderuns.list()` to watch all CodeRuns
- Kubernetes API returns JSON with ALL CodeRuns
- Serde tries to deserialize the entire list
- **ONE CodeRun with empty cliType breaks deserialization**
- **Controller cannot reconcile ANY CodeRuns** (not just the broken one!)
- All workflows stall, no new jobs start

## Observed Behavior

**Symptoms:**
- CodeRun exists for 40+ minutes with no status
- No pod created
- Workflow stuck waiting: "Neither success condition nor failure condition matched"
- Controller logs: SerdeError every ~50 seconds
- **All other CodeRuns also blocked** from reconciliation

**Example:** Task 7 Rex CodeRun created, but controller couldn't reconcile it because Task 5 security CodeRun was broken.

## Fixes Implemented

### ✅ Fix #1: MCP Tool Provides security-agent (DONE)

**Commits:**
- `1666a612` - feat(cipher): add security agent parameters to MCP play tool
- `8655956f` - fix: Add ready-for-qa sensor and improve Factory/Toolman resilience

**Status:** Merged to main Oct 17, 2025

**Prevention:** All new workflows include `security-agent` parameter from config

### ✅ Fix #2: ConfigMap Cleanup Race Condition (DONE)

**Issue:** ConfigMaps deleted while pods still running
**Fix:** Check pod status before deleting ConfigMaps
**PR:** #1157 
**File:** `controller/src/tasks/code/resources.rs`

### ✅ Fix #3: Manual Cleanup of Broken CodeRuns (TEMPORARY)

**Action:** Delete CodeRuns with empty cliType
```bash
kubectl delete coderun cto-parallel-test-t2-security-nrmnn -n agent-platform
kubectl delete coderun cto-parallel-test-t5-security-crnvp -n agent-platform  
kubectl delete coderun cto-parallel-test-t7-security-hdzg9 -n agent-platform
```

**Result:** Controller immediately resumed reconciliation

## Fixes Still Needed

### ❌ Fix #4: Workflow Parameter Validation (HIGH PRIORITY)

**Add validation step before creating CodeRun:**

```yaml
steps:
  - - name: validate-required-params
      script:
        image: alpine:3.20
        source: |
          #!/bin/sh
          set -e
          
          GITHUB_APP="{{inputs.parameters.github-app}}"
          CLI_TYPE="{{inputs.parameters.cli-type}}"
          MODEL="{{inputs.parameters.model}}"
          STAGE="{{inputs.parameters.stage}}"
          
          if [ -z "$GITHUB_APP" ] || [ "$GITHUB_APP" = "null" ]; then
            echo "❌ ERROR: github-app parameter is empty for stage: $STAGE"
            echo "This would create a broken CodeRun that crashes the controller!"
            exit 1
          fi
          
          if [ -z "$CLI_TYPE" ] || [ "$CLI_TYPE" = "null" ]; then
            echo "❌ ERROR: cli-type parameter is empty for stage: $STAGE"  
            exit 1
          fi
          
          echo "✅ Parameters validated for stage: $STAGE"
  - - name: create-coderun-resource
      # ... existing resource creation
```

**Benefit:** Workflow fails fast with clear error instead of creating broken CodeRun

### ❌ Fix #5: CodeRun CRD Validation Webhook (MEDIUM PRIORITY)

**Add ValidatingWebhookConfiguration:**

```yaml
apiVersion: admissionregistration.k8s.io/v1
kind: ValidatingWebhookConfiguration
metadata:
  name: coderun-validator
webhooks:
- name: validate.coderun.agents.platform
  rules:
  - operations: ["CREATE", "UPDATE"]
    apiGroups: ["agents.platform"]
    apiVersions: ["v1"]
    resources: ["coderuns"]
  failurePolicy: Fail
  sideEffects: None
  admissionReviewVersions: ["v1"]
  clientConfig:
    service:
      name: controller
      namespace: agent-platform
      path: "/validate-coderun"
```

**Controller validation logic:**
```rust
fn validate_coderun(cr: &CodeRun) -> Result<(), String> {
    if cr.spec.github_app.as_ref().map_or(true, |s| s.is_empty()) {
        return Err("githubApp cannot be empty".to_string());
    }
    
    // Note: Can't validate cliType directly due to deserialize happening first
    // This would require making CLIType Optional or using raw JSON validation
    
    Ok(())
}
```

**Benefit:** Kubernetes rejects broken CodeRuns before they're created

### ❌ Fix #6: Make CLIType Optional (COMPLEX - DEFERRED)

**Change:**
```rust
pub struct CLIConfig {
    #[serde(rename = "cliType", default)]
    pub cli_type: Option<CLIType>,  // Now optional
```

**Required Changes:**
- Update ~10 files that access `cli_type`
- Add fallback logic for missing cli_type
- Handle None case throughout codebase
- Test all CLI adapters

**Complexity:** High - touches many files
**Priority:** Low - prevented by fixes #4 and #5

## Recommended Action Plan

**Immediate (Now):**
1. ✅ Clean up broken CodeRuns manually (done)
2. ✅ Ensure MCP in main has security-agent (done)

**Short Term (Next PR):**
3. ❌ Add parameter validation to workflow template (Fix #4)
4. ❌ Document in README/runbook for operators

**Medium Term:**
5. ❌ Add CodeRun validation webhook (Fix #5)
6. ❌ Add monitoring/alerts for CodeRuns stuck without status

**Long Term (Optional):**
7. ❌ Refactor CLIType to be Optional (Fix #6)
8. ❌ Make controller list operations more resilient (skip malformed items)

## Prevention Checklist

**For Operators:**
- [ ] Always use latest MCP binary
- [ ] Verify cto-config.json has securityAgent defined
- [ ] Check workflows have all agent parameters before submitting
- [ ] Monitor for CodeRuns with no status > 5 minutes
- [ ] Delete any CodeRuns with empty githubApp immediately

**For Developers:**
- [ ] MCP tool must provide ALL agent parameters (implementation, quality, security, testing)
- [ ] Workflow template should validate parameters before CodeRun creation
- [ ] Controller should log warnings for CodeRuns with missing required fields
- [ ] Consider CRD validation webhook for production

## Related Issues

- Empty `githubApp` → Controller can't determine which GitHub App secret to use
- Empty `cliType` → Deserialization fails, breaks controller list operations  
- Empty `model` → Even if pod created, agent execution would fail

All three must be present and valid for CodeRun to work.

