# Autonomous Implementation Prompt: PR Validation DAG with Compliance Gates

## Mission Statement

You are implementing a production-ready PR validation system that enforces zero-warning code quality through a structured Clippy → QA pipeline with independent compliance verification. Your goal is to create a robust DAG workflow that prevents code quality regressions while providing comprehensive Kubernetes-based evidence of testing.

## Context

This system replaces ad-hoc code quality checks with a systematic, gated approach. The implementation must handle multiple programming languages, enforce strict compliance rules, and provide tamper-proof evidence of quality assurance testing in Kubernetes environments.

## Technical Requirements

### Must Implement

1. **DAG WorkflowTemplate Structure**
   - Four-step pipeline: clippy-format → qa-testing → verify-compliance + verify-k8s-proof → gates-passed
   - Parallel execution of compliance and proof verification after their respective dependencies
   - Failure of any gate prevents overall success

2. **Multi-Language Compliance Verification**
   - Rust: cargo fmt --check + cargo clippy with -D warnings -W clippy::pedantic
   - Node.js: prettier --check + eslint --max-warnings=0
   - Go: gofmt + golangci-lint with strict configuration
   - Python: black --check + ruff check --no-cache
   - Shell: shellcheck for script validation
   - Auto-detection via file presence (Cargo.toml, package.json, go.mod, pyproject.toml)

3. **QA Evidence Collection System**
   - Required artifacts: summary.json, kubectl logs, kubectl resources, test evidence
   - Structured evidence validation with schema enforcement
   - Kubernetes cluster state capture for audit trail

4. **Independent Gate Enforcement**
   - verify-compliance runs independently of agent execution
   - verify-k8s-proof validates QA artifact completeness and structure
   - Both gates required before gates-passed step executes

5. **Configuration Override System**
   - Support .pr-validation.yml for project-specific customization
   - Override language detection, commands, working directory
   - Maintain security through validated configuration

### DAG Template Structure

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: pr-validation
spec:
  entrypoint: dag
  arguments:
    parameters:
      - name: owner
      - name: repo  
      - name: prNumber
      - name: ref
      - name: event
  templates:
    - name: dag
      dag:
        tasks:
          - name: clippy-format
            templateRef: {name: coderun-template, template: coderun-main}
            arguments:
              parameters:
                - {name: github-app, value: clippy}
                - {name: event, value: "{{workflow.arguments.parameters.event}}"}
          - name: qa-testing
            dependencies: [clippy-format]
            templateRef: {name: coderun-template, template: coderun-main}
            arguments:
              parameters:
                - {name: github-app, value: qa}
                - {name: event, value: "{{workflow.arguments.parameters.event}}"}
          - name: verify-compliance
            dependencies: [clippy-format]
            template: verify-compliance
          - name: verify-k8s-proof
            dependencies: [qa-testing]
            template: verify-k8s-proof
          - name: gates-passed
            dependencies: [verify-compliance, verify-k8s-proof]
            template: gates-passed
```

### Compliance Verification Implementation

**Container Image Requirements:**
- Base: ghcr.io/myorg/ci-multilang:latest
- Include: bash, git, jq, yq, rustup/cargo, node/npm, go, python3/pip, shellcheck
- Security: non-root user, minimal attack surface

**Language Detection Logic:**
```bash
detect_language() {
  if [ -f .pr-validation.yml ]; then
    LANGUAGE=$(yq eval '.language' .pr-validation.yml)
  elif [ -f Cargo.toml ]; then
    LANGUAGE="rust"
  elif [ -f package.json ]; then
    LANGUAGE="node"  
  elif [ -f go.mod ]; then
    LANGUAGE="go"
  elif [ -f pyproject.toml ] || [ -f requirements.txt ]; then
    LANGUAGE="python"
  else
    LANGUAGE="shell"
  fi
}
```

**Validation Commands by Language:**
```bash
validate_rust() {
  rustup component add rustfmt clippy || true
  cargo fmt --all -- --check || return 1
  cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic || return 1
}

validate_node() {
  npm ci || return 1
  npx prettier --check . || return 1
  npx eslint . --max-warnings=0 || return 1
}

validate_go() {
  test -z "$(gofmt -l .)" || { echo 'gofmt issues found'; return 1; }
  golangci-lint run --timeout 5m || return 1
}

validate_python() {
  pip install -U pip black ruff || return 1
  black --check . || return 1
  ruff check --no-cache || return 1
}
```

**Compliance Summary Generation:**
```bash
generate_compliance_summary() {
  local language=$1 fmt_status=$2 lint_status=$3 warnings_count=$4
  
  jq -n \
    --arg language "$language" \
    --argjson tools "$(get_tool_versions)" \
    --arg fmt_status "$fmt_status" \
    --arg lint_status "$lint_status" \
    --argjson warnings_count "$warnings_count" \
    --arg timestamp "$(date -Iseconds)" \
    '{
      language: $language,
      tools: $tools,
      fmtStatus: $fmt_status,
      lintStatus: $lint_status,
      warningsCount: $warnings_count,
      timestamp: $timestamp
    }' > /artifacts/compliance/summary.json
}
```

### QA Evidence Schema Implementation

**Required Directory Structure:**
```bash
setup_qa_artifacts() {
  mkdir -p "$ARTIFACTS_DIR"/{logs,k8s,evidence}
  
  # Ensure summary.json exists with proper schema
  if [ ! -f "$ARTIFACTS_DIR/summary.json" ]; then
    echo "ERROR: QA must generate summary.json" >&2
    return 1
  fi
}
```

**Evidence Validation Logic:**
```bash
validate_qa_evidence() {
  local summary_file="$1/summary.json"
  
  # Schema validation
  jq -e '.cluster and .namespace and .testCases and .evidence' "$summary_file" >/dev/null || {
    echo "ERROR: Invalid summary.json schema" >&2
    return 1
  }
  
  # Validate test cases
  local failed_tests=$(jq -r '.testCases[] | select(.status != "passed") | .name' "$summary_file")
  if [ -n "$failed_tests" ]; then
    echo "ERROR: Failed test cases: $failed_tests" >&2
    return 1
  fi
  
  # Validate evidence files exist
  while read -r evidence_path; do
    [ -f "$ARTIFACTS_DIR/$evidence_path" ] || {
      echo "ERROR: Evidence file missing: $evidence_path" >&2
      return 1
    }
  done < <(jq -r '.evidence[].path' "$summary_file")
}
```

### Kubernetes Evidence Collection

**kubectl Operations:**
```bash
collect_k8s_evidence() {
  local namespace=$(jq -r '.namespace' "$ARTIFACTS_DIR/summary.json")
  
  # Collect resources
  kubectl get pods -n "$namespace" -o yaml > "$ARTIFACTS_DIR/k8s/pods.yaml"
  kubectl get services -n "$namespace" -o yaml > "$ARTIFACTS_DIR/k8s/services.yaml"  
  kubectl get events -n "$namespace" --sort-by=.lastTimestamp -o yaml > "$ARTIFACTS_DIR/k8s/events.yaml"
  
  # Collect logs
  for pod in $(kubectl get pods -n "$namespace" -o name); do
    pod_name=${pod#*/}
    kubectl logs "$pod" -n "$namespace" --all-containers=true > "$ARTIFACTS_DIR/logs/${pod_name}.log" || true
  done
}
```

**Verification Output Generation:**
```bash
generate_verification_result() {
  local summary_file="$ARTIFACTS_DIR/summary.json"
  local cluster=$(jq -r '.cluster' "$summary_file")
  local namespace=$(jq -r '.namespace' "$summary_file")
  local test_count=$(jq -r '.testCases | length' "$summary_file")
  local passed_count=$(jq -r '[.testCases[] | select(.status == "passed")] | length' "$summary_file")
  
  jq -n \
    --arg valid "true" \
    --argjson missing_evidence "[]" \
    --argjson counts "{\"tests\": $test_count, \"passed\": $passed_count, \"failed\": $((test_count - passed_count))}" \
    --arg timestamp "$(date -Iseconds)" \
    '{
      valid: $valid,
      missingEvidence: $missing_evidence,
      counts: $counts,
      timestamp: $timestamp
    }' > "$ARTIFACTS_DIR/verification.json"
}
```

## Implementation Approach

### Phase 1: DAG Template Creation

1. **Create pr-validation.yaml with proper DAG structure**
2. **Wire dependencies: clippy-format → qa-testing, parallel gates**  
3. **Add gates-passed final step with both gate dependencies**
4. **Test template validation and submission**

### Phase 2: Multi-Language Compliance Container

1. **Build ci-multilang container with all required tools**
2. **Implement language detection and validation scripts**
3. **Add configuration override support via .pr-validation.yml**
4. **Test all language validation paths**

### Phase 3: QA Evidence System

1. **Define evidence directory structure and schemas**
2. **Implement evidence collection scripts for kubernetes resources**
3. **Create validation logic for summary.json and evidence files**
4. **Test end-to-end evidence generation and validation**

### Phase 4: Gate Integration and Testing

1. **Integrate verify-compliance and verify-k8s-proof templates**
2. **Test failure conditions and error handling**
3. **Validate parallel execution and dependency management**
4. **Implement comprehensive end-to-end testing**

## Validation Requirements

### Compliance Gate Testing

**Must Test:**
- Rust project with warnings → compliance failure
- Node.js project with eslint errors → compliance failure  
- Go project with gofmt issues → compliance failure
- Python project with black formatting issues → compliance failure
- Clean project in all languages → compliance success

**Test Commands:**
```bash
# Test Rust compliance failure
echo "fn main() { let x = 1; }" > src/main.rs  # unused variable
argo submit --from workflowtemplate/pr-validation -p github-app=clippy --watch

# Test compliance success
cargo fmt && cargo clippy --fix
argo submit --from workflowtemplate/pr-validation -p github-app=clippy --watch
```

### QA Evidence Testing

**Must Test:**
- Missing summary.json → proof validation failure
- Invalid summary.json schema → proof validation failure
- Missing evidence files → proof validation failure  
- Complete evidence set → proof validation success

**Test Setup:**
```bash
# Test missing evidence
mkdir -p /artifacts/qa/proof/logs
# Don't create summary.json
argo submit --from workflowtemplate/pr-validation -p github-app=qa --watch

# Test complete evidence
cat > /artifacts/qa/proof/summary.json <<EOF
{
  "cluster": "test-cluster",
  "namespace": "pr-123",
  "testCases": [{"name": "test1", "status": "passed"}],
  "evidence": [{"path": "logs/test.log", "type": "log"}]
}
EOF
touch /artifacts/qa/proof/logs/test.log
```

### Integration Testing

**End-to-End Scenarios:**
1. Complete success path: all gates pass
2. Compliance failure: gates-passed never executes
3. QA proof failure: gates-passed never executes
4. Parallel execution: both gates run simultaneously after dependencies

## Security Requirements

### Container Security
- Run as non-root user (65532:65532)
- Read-only root filesystem where possible
- Drop all capabilities except necessary ones
- Use distroless or minimal base images

### RBAC Configuration
```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: pr-validation-role
rules:
- apiGroups: [""]
  resources: ["pods", "services", "events"]
  verbs: ["get", "list"]
- apiGroups: [""]
  resources: ["pods/log"]
  verbs: ["get"]
```

### Evidence Integrity
- All evidence files must be tamper-evident
- Summary.json includes checksums of evidence files
- Verification step validates all checksums
- Audit trail of all validation activities

## Performance Requirements

### Resource Limits
```yaml
resources:
  limits:
    cpu: 2
    memory: 4Gi
  requests:
    cpu: 100m
    memory: 512Mi
```

### Execution Time Targets
- Compliance verification: < 5 minutes for typical project
- QA evidence validation: < 2 minutes
- Total DAG execution: < 30 minutes including agent steps
- Parallel gate execution reduces total time vs sequential

## Error Handling Requirements

### Compliance Failures
```bash
# Clear error messages with remediation guidance
"Compliance failed: 3 clippy warnings found. Run 'cargo clippy --fix' to resolve."
"Format check failed: 5 files need formatting. Run 'cargo fmt' to fix."
"ESLint failed: 2 errors, 1 warning. Run 'npm run lint:fix' to resolve."
```

### Evidence Failures
```bash
"QA proof validation failed: summary.json not found at /artifacts/qa/proof/summary.json"
"Evidence file missing: logs/api-server.log referenced in summary.json but not present"
"Invalid test status in summary.json: 'unknown' not in [passed, failed, skipped]"
```

## Success Criteria

Your implementation is complete when:

1. **DAG Structure**: Four-step pipeline with correct dependencies and parallel gates
2. **Multi-Language Support**: All specified languages validate correctly with zero warnings
3. **Evidence System**: Complete Kubernetes evidence collection and validation
4. **Gate Enforcement**: Both compliance and proof gates required for success
5. **Error Handling**: Clear error messages with actionable remediation guidance
6. **Configuration**: Project override system works via .pr-validation.yml
7. **Testing**: Comprehensive test suite covers all failure and success scenarios
8. **Security**: RBAC, container security, and evidence integrity implemented

## Delivery Artifacts

Create these files:
- pr-validation.yaml - Complete DAG WorkflowTemplate
- ci-multilang Dockerfile and scripts
- kubectl-jq Dockerfile for evidence validation
- Test cases for all languages and failure modes
- Configuration examples and documentation
- RBAC manifests and security configurations

Remember: Quality gates must be uncompromising. The system should fail fast and provide clear guidance for remediation, but never allow warnings or missing evidence to pass through the pipeline.