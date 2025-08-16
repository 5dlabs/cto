# Toolman Guide: PR Validation DAG with Compliance Gates

## Overview

This guide provides comprehensive instructions for using the PR validation DAG system that enforces code quality through a structured Clippy → QA pipeline with independent compliance gates. The system ensures zero warnings and provides Kubernetes-based evidence of quality assurance.

## Available Tools

### 1. PR Validation DAG (`pr-validation-dag`)

**Purpose**: Execute comprehensive PR validation workflow with compliance gates and QA verification.

**Base URL**: `https://argo-workflows.workflows.svc.cluster.local:2746`

#### Submit PR Validation Workflow

```bash
# Submit validation for specific PR
argo submit --from workflowtemplate/pr-validation \
  -p owner=myorg \
  -p repo=myrepo \
  -p prNumber=123 \
  -p ref=feature-branch \
  -p event='{"pull_request":{"number":123}}' \
  --watch

# Submit with minimal parameters (auto-detection)
argo submit --from workflowtemplate/pr-validation \
  -p event='{"repository":{"owner":{"login":"myorg"},"name":"myrepo"},"pull_request":{"number":123}}' \
  --watch
```

#### Monitor Workflow Progress

```bash
# Watch workflow execution
argo watch pr-validation-abc123

# Get detailed workflow status
argo get pr-validation-abc123

# Check specific step logs
argo logs pr-validation-abc123 -c verify-compliance
argo logs pr-validation-abc123 -c verify-k8s-proof
```

#### View Gate Results

```bash
# Check compliance gate artifacts
argo get pr-validation-abc123 -o json | jq '.status.nodes[] | select(.displayName=="verify-compliance") | .outputs.artifacts'

# Check QA proof validation results
argo get pr-validation-abc123 -o json | jq '.status.nodes[] | select(.displayName=="verify-k8s-proof") | .outputs.artifacts'
```

### 2. Compliance Verification API (`compliance-verification-api`)

**Purpose**: Multi-language compliance verification with zero-warning enforcement.

**Base URL**: `http://compliance-verifier.workflows.svc.cluster.local:8080`

#### Verify Code Compliance

```http
POST /verify
Content-Type: application/json

{
  "language": "rust",
  "workspaceDir": "/workspace/src",
  "config": {
    "fmtCommand": "cargo fmt --all -- --check",
    "lintCommand": "cargo clippy --workspace -- -D warnings"
  }
}
```

**Response**:
```json
{
  "language": "rust",
  "tools": [
    {"name": "rustfmt", "version": "1.7.0"},
    {"name": "clippy", "version": "1.75.0"}
  ],
  "fmtStatus": "passed",
  "lintStatus": "passed",
  "warningsCount": 0,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Usage Examples**:

```bash
# Test Rust project compliance
curl -X POST http://compliance-verifier:8080/verify \
  -H "Content-Type: application/json" \
  -d '{
    "language": "rust",
    "workspaceDir": "/workspace/rust-project"
  }'

# Test Node.js project with custom config
curl -X POST http://compliance-verifier:8080/verify \
  -H "Content-Type: application/json" \
  -d '{
    "language": "node",
    "workspaceDir": "/workspace/node-app",
    "config": {
      "fmtCommand": "npx prettier --check src/",
      "lintCommand": "npx eslint src/ --max-warnings=0"
    }
  }'
```

#### Auto-Detect Language

```bash
# Detect project language from file patterns
curl -X POST http://compliance-verifier:8080/detect-language \
  -H "Content-Type: application/json" \
  -d '{"workspaceDir": "/workspace/project"}'
```

**Response**:
```json
{
  "detectedLanguage": "rust",
  "confidence": "high",
  "indicators": ["Cargo.toml", "src/main.rs"],
  "alternativeLanguages": []
}
```

#### Validate Configuration

```bash
# Validate .pr-validation.yml file
curl -X POST http://compliance-verifier:8080/validate-config \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "language": "rust",
      "fmtCommand": "cargo fmt --check",
      "lintCommand": "cargo clippy -- -D warnings"
    }
  }'
```

### 3. QA Evidence Validator (`qa-evidence-validator`)

**Purpose**: Collect and validate Kubernetes evidence for QA verification.

#### Collect Kubernetes Evidence

```http
POST /collect-evidence
Content-Type: application/json

{
  "namespace": "pr-123-test",
  "outputDir": "/artifacts/qa/proof"
}
```

**Usage Example**:
```bash
# Collect evidence from test namespace
curl -X POST http://qa-evidence:8080/collect-evidence \
  -H "Content-Type: application/json" \
  -d '{
    "namespace": "pr-123-test",
    "outputDir": "/artifacts/qa/proof"
  }'
```

#### Validate Evidence Completeness

```bash
# Validate collected evidence
curl -X POST http://qa-evidence:8080/validate-evidence \
  -H "Content-Type: application/json" \
  -d '{
    "evidenceDir": "/artifacts/qa/proof",
    "summaryFile": "/artifacts/qa/proof/summary.json"
  }'
```

**Response**:
```json
{
  "valid": true,
  "missingEvidence": [],
  "counts": {
    "tests": 15,
    "passed": 15,
    "failed": 0
  },
  "evidenceFiles": [
    "logs/api-server.log",
    "k8s/pods.yaml",
    "k8s/services.yaml"
  ],
  "timestamp": "2024-01-15T10:35:00Z"
}
```

### 4. Kubernetes API (`kubernetes-api`)

**Purpose**: Direct Kubernetes API access for resource inspection and log collection.

#### Common Operations

```bash
# List pods in test namespace
kubectl get pods -n pr-123-test -o yaml

# Get pod logs for evidence collection
kubectl logs pod-name -n pr-123-test --all-containers=true

# Get services and events
kubectl get services -n pr-123-test -o yaml
kubectl get events -n pr-123-test --sort-by=.lastTimestamp
```

## Local Development Tools

### Compliance Tester

**Purpose**: Test multi-language compliance validation locally.

**Command**: `./scripts/test-compliance.sh --languages rust,node,go,python --test-dir ./test-projects`

#### Test Project Structure

```
test-projects/
├── rust-clean/          # Rust project with no warnings
├── rust-warnings/       # Rust project with clippy warnings  
├── node-clean/          # Node.js project with no eslint issues
├── node-warnings/       # Node.js project with eslint warnings
├── go-formatted/        # Go project properly formatted
├── go-unformatted/      # Go project needing gofmt
├── python-black/        # Python project formatted with black
└── python-unformatted/  # Python project needing formatting
```

#### Usage Examples

```bash
# Test all languages with comprehensive scenarios
./scripts/test-compliance.sh

# Test specific language
./scripts/test-compliance.sh --language rust --project rust-warnings

# Test with custom container image
CI_MULTILANG_IMAGE=ghcr.io/myorg/ci-multilang:v2.0.0 \
  ./scripts/test-compliance.sh --language node

# Dry run without actual validation
./scripts/test-compliance.sh --dry-run --verbose
```

#### Configuration File (`.test-compliance.config`)

```yaml
defaultLanguages: ["rust", "node", "go", "python"]
testProjects:
  rust:
    clean: "./test-projects/rust-clean"
    warnings: "./test-projects/rust-warnings"
  node:
    clean: "./test-projects/node-clean"  
    warnings: "./test-projects/node-warnings"
containerImage: "ghcr.io/myorg/ci-multilang:latest"
timeout: 300
verbose: true
```

### Evidence Simulator

**Purpose**: Generate realistic QA evidence for testing validation logic.

**Command**: `./scripts/simulate-qa-evidence.sh --namespace test-pr-123 --output-dir ./test-evidence`

#### Generate Complete Evidence Set

```bash
# Generate full evidence structure
./scripts/simulate-qa-evidence.sh \
  --namespace test-pr-123 \
  --output-dir ./test-evidence \
  --scenario complete

# Generate evidence with missing files (for failure testing)
./scripts/simulate-qa-evidence.sh \
  --namespace test-pr-123 \
  --output-dir ./test-evidence \
  --scenario missing-logs

# Generate evidence with failed test cases
./scripts/simulate-qa-evidence.sh \
  --namespace test-pr-123 \
  --output-dir ./test-evidence \
  --scenario failed-tests
```

#### Evidence Scenarios

**Complete Evidence**:
```
test-evidence/
├── summary.json         # All tests passed
├── logs/
│   ├── api-server.log   # Complete application logs
│   └── worker.log       # Background job logs
├── k8s/
│   ├── pods.yaml        # Pod configurations
│   ├── services.yaml    # Service definitions
│   └── events.yaml      # Cluster events
└── evidence/
    ├── test-results.xml # Test execution results
    └── coverage.json    # Code coverage report
```

### DAG Validator

**Purpose**: Validate PR validation DAG structure and execution paths.

**Command**: `./scripts/validate-pr-dag.sh --template pr-validation.yaml --test-scenarios all`

#### Validation Scenarios

```bash
# Validate DAG structure and dependencies
./scripts/validate-pr-dag.sh --template pr-validation.yaml --check-structure

# Test all failure scenarios
./scripts/validate-pr-dag.sh --test-scenarios failure-modes

# Test parallel execution
./scripts/validate-pr-dag.sh --test-scenarios parallel-gates

# Validate with real PR data
./scripts/validate-pr-dag.sh \
  --template pr-validation.yaml \
  --pr-data ./test-data/pr-123-event.json \
  --dry-run
```

## Configuration Examples

### Project Configuration (.pr-validation.yml)

#### Rust Project Configuration

```yaml
language: rust
fmtCommand: "cargo fmt --all -- --check"
lintCommand: "cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic"
workingDir: "."
timeout: 600
allowedWarnings: 0
customTools:
  - name: "cargo-audit"
    command: "cargo audit --deny warnings"
    required: false
```

#### Node.js Monorepo Configuration

```yaml
language: node
fmtCommand: "npm run format:check"
lintCommand: "npm run lint:ci"
workingDir: "packages/api"
timeout: 300
customSteps:
  - name: "type-check"
    command: "npm run type-check"
    required: true
skipSteps: ["shellcheck"]
```

#### Go Project Configuration

```yaml
language: go
fmtCommand: "gofmt -l . | tee /dev/stderr | wc -l | xargs test 0 -eq"
lintCommand: "golangci-lint run --timeout 10m"
workingDir: "."
customChecks:
  - name: "go-mod-tidy"
    command: "go mod tidy && git diff --exit-code go.mod go.sum"
    description: "Ensure go.mod and go.sum are tidy"
```

### QA Evidence Configuration

#### Complete summary.json Example

```json
{
  "cluster": "dev-cluster-01",
  "namespace": "pr-123-test",
  "testCases": [
    {
      "name": "unit-tests",
      "status": "passed",
      "details": "All 45 unit tests passed successfully",
      "duration": 120,
      "coverage": 85.5
    },
    {
      "name": "integration-tests",
      "status": "passed", 
      "details": "API integration tests completed",
      "duration": 300,
      "endpoints": 12
    },
    {
      "name": "e2e-tests",
      "status": "passed",
      "details": "End-to-end UI automation successful",
      "duration": 450,
      "scenarios": 8
    }
  ],
  "evidence": [
    {
      "path": "logs/api-server.log",
      "type": "log",
      "size": 1024576,
      "checksum": "sha256:abc123..."
    },
    {
      "path": "k8s/pods.yaml",
      "type": "kubernetes-resource",
      "resourceCount": 3
    },
    {
      "path": "evidence/test-results.xml",
      "type": "test-results",
      "format": "junit"
    }
  ],
  "environment": {
    "kubernetesVersion": "1.30.0",
    "clusterName": "dev-cluster-01",
    "region": "us-west-2"
  },
  "metadata": {
    "generatedAt": "2024-01-15T10:30:00Z",
    "generatedBy": "qa-agent-v2.1.0",
    "prNumber": 123,
    "commitSha": "abc123def456"
  }
}
```

## Common Usage Patterns

### 1. Automated PR Validation

**Argo Events Sensor Integration**:
```yaml
triggers:
  - template:
      name: pr-validation-trigger
      k8s:
        operation: create
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: pr-validation-
            spec:
              workflowTemplateRef:
                name: pr-validation
              arguments:
                parameters:
                  - name: event
                    value: "{{events.github.body}}"
```

### 2. Manual Validation Testing

```bash
#!/bin/bash
# manual-pr-validation.sh
OWNER=${1:-myorg}
REPO=${2:-myrepo}
PR_NUMBER=${3:-123}
REF=${4:-main}

echo "Starting PR validation for $OWNER/$REPO #$PR_NUMBER"

# Create event payload
EVENT_JSON=$(jq -n \
  --arg owner "$OWNER" \
  --arg repo "$REPO" \
  --arg pr "$PR_NUMBER" \
  --arg ref "$REF" \
  '{
    repository: {
      owner: {login: $owner},
      name: $repo,
      full_name: ($owner + "/" + $repo)
    },
    pull_request: {
      number: ($pr | tonumber),
      head: {ref: $ref}
    }
  }')

# Submit workflow
WORKFLOW_NAME=$(argo submit --from workflowtemplate/pr-validation \
  -p owner="$OWNER" \
  -p repo="$REPO" \
  -p prNumber="$PR_NUMBER" \
  -p ref="$REF" \
  -p event="$EVENT_JSON" \
  -o name)

echo "Submitted workflow: $WORKFLOW_NAME"
echo "Monitor with: argo watch $WORKFLOW_NAME"

# Watch execution
argo watch "$WORKFLOW_NAME"
```

### 3. Compliance Testing Script

```bash
#!/bin/bash
# test-compliance.sh
PROJECT_DIR=${1:-.}
LANGUAGE=${2:-auto}

echo "Testing compliance for $PROJECT_DIR (language: $LANGUAGE)"

# Auto-detect language if not specified
if [ "$LANGUAGE" = "auto" ]; then
  if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
    LANGUAGE="rust"
  elif [ -f "$PROJECT_DIR/package.json" ]; then
    LANGUAGE="node"
  elif [ -f "$PROJECT_DIR/go.mod" ]; then
    LANGUAGE="go"
  elif [ -f "$PROJECT_DIR/pyproject.toml" ]; then
    LANGUAGE="python"
  else
    LANGUAGE="shell"
  fi
  echo "Detected language: $LANGUAGE"
fi

# Run compliance check
docker run --rm -v "$PROJECT_DIR:/workspace" \
  -e LANGUAGE="$LANGUAGE" \
  -e WORKSPACE_DIR="/workspace" \
  ghcr.io/myorg/ci-multilang:latest \
  /scripts/verify-compliance.sh

echo "Compliance check completed"
```

### 4. Evidence Collection Script

```bash
#!/bin/bash  
# collect-qa-evidence.sh
NAMESPACE=${1:-default}
OUTPUT_DIR=${2:-./evidence}

echo "Collecting QA evidence from namespace: $NAMESPACE"

mkdir -p "$OUTPUT_DIR"/{logs,k8s,evidence}

# Collect Kubernetes resources
kubectl get pods -n "$NAMESPACE" -o yaml > "$OUTPUT_DIR/k8s/pods.yaml"
kubectl get services -n "$NAMESPACE" -o yaml > "$OUTPUT_DIR/k8s/services.yaml"
kubectl get events -n "$NAMESPACE" --sort-by=.lastTimestamp -o yaml > "$OUTPUT_DIR/k8s/events.yaml"

# Collect pod logs
for pod in $(kubectl get pods -n "$NAMESPACE" -o name); do
  pod_name=${pod#*/}
  kubectl logs "$pod" -n "$NAMESPACE" --all-containers=true > "$OUTPUT_DIR/logs/${pod_name}.log" || true
done

# Generate summary
cat > "$OUTPUT_DIR/summary.json" <<EOF
{
  "cluster": "$(kubectl config current-context)",
  "namespace": "$NAMESPACE",
  "testCases": [
    {
      "name": "evidence-collection",
      "status": "passed",
      "details": "Successfully collected Kubernetes evidence"
    }
  ],
  "evidence": [
    $(find "$OUTPUT_DIR" -name "*.yaml" -o -name "*.log" | sed 's|'"$OUTPUT_DIR"'/||' | jq -R -s 'split("\n") | map(select(length > 0)) | map({path: ., type: "file"})')
  ]
}
EOF

echo "Evidence collection completed in $OUTPUT_DIR"
```

## Troubleshooting

### Common Issues

#### 1. Compliance Verification Failures

**Symptoms**:
- verify-compliance step fails with tool errors
- "command not found" errors in compliance logs

**Diagnosis**:
```bash
# Check container image tools
kubectl run debug --rm -it --image=ghcr.io/myorg/ci-multilang:latest -- which cargo
kubectl run debug --rm -it --image=ghcr.io/myorg/ci-multilang:latest -- rustc --version

# Test language detection
echo '{"workspaceDir": "/workspace/project"}' | \
  curl -X POST http://compliance-verifier:8080/detect-language \
  -H "Content-Type: application/json" -d @-
```

**Solutions**:
- Update ci-multilang container image with missing tools
- Check .pr-validation.yml configuration syntax
- Verify project structure matches expected language patterns

#### 2. QA Evidence Collection Issues

**Symptoms**:
- verify-k8s-proof fails with missing evidence
- kubectl commands fail with permission errors

**Diagnosis**:
```bash
# Check QA evidence directory
argo logs workflow-name -c qa-testing | grep "artifacts/qa/proof"

# Test kubectl permissions
kubectl auth can-i get pods --as=system:serviceaccount:workflows:pr-validation-sa

# Validate evidence structure
find /artifacts/qa/proof -type f -exec ls -la {} \;
```

**Solutions**:
- Fix RBAC permissions for evidence collection
- Ensure QA agent generates required summary.json
- Check namespace exists and is accessible

#### 3. DAG Execution Problems

**Symptoms**:
- Gates don't execute in parallel
- gates-passed runs when it shouldn't

**Diagnosis**:
```bash
# Check DAG structure
argo get workflow-name -o json | jq '.status.nodes[] | {name: .displayName, phase: .phase, deps: .dependencies}'

# Verify template structure
argo template get pr-validation -o yaml | grep -A20 "dag:"
```

**Solutions**:
- Verify DAG dependencies in template
- Check Argo Workflows version compatibility
- Review template lint results

### Debug Commands

```bash
# Monitor workflow execution with detailed logs
argo watch workflow-name --log

# Check specific step artifacts
argo get workflow-name -o json | jq '.status.nodes[] | select(.displayName=="verify-compliance") | .outputs'

# Test compliance locally
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/myorg/ci-multilang:latest \
  /bin/bash -c 'cd /workspace && /scripts/verify-compliance.sh'

# Validate QA evidence
cat /artifacts/qa/proof/summary.json | jq '.testCases[] | select(.status != "passed")'

# Check gate dependencies
argo get workflow-name -o json | jq '.spec.templates[] | select(.name=="dag") | .dag.tasks[] | {name, dependencies}'
```

### Performance Optimization

```bash
# Monitor resource usage
kubectl top pods -n workflows --sort-by=cpu

# Check workflow queue
argo list --status Pending --sort creationTimestamp

# Optimize container resources
kubectl get workflows -o json | jq '.items[] | {name: .metadata.name, resources: .spec.templates[].container.resources}'

# Monitor artifact size
find /artifacts -name "*.json" -exec du -h {} \;
```

## Best Practices

### Code Quality Configuration

1. **Zero Warning Policy**: Always configure tools to fail on any warnings
2. **Version Pinning**: Pin tool versions in container images for consistency
3. **Custom Rules**: Use .pr-validation.yml for project-specific requirements
4. **Fast Feedback**: Configure tools for quick execution on typical projects

### Evidence Collection

1. **Complete Evidence**: Always collect logs, resources, and test results
2. **Structured Data**: Use consistent JSON schemas for evidence
3. **Checksums**: Include file checksums for integrity verification
4. **Retention**: Configure appropriate retention policies for evidence

### Performance Optimization

1. **Parallel Execution**: Leverage DAG parallelism for independent tasks
2. **Resource Limits**: Set appropriate CPU/memory limits
3. **Caching**: Use tool caches where possible to speed up validation
4. **Incremental Validation**: Validate only changed files when feasible

### Security Considerations

1. **Minimal Permissions**: Use least-privilege RBAC for all operations
2. **Container Security**: Run containers as non-root with read-only filesystems
3. **Evidence Integrity**: Implement checksums and audit trails
4. **Secret Management**: Never include secrets in evidence or logs

For additional support and advanced configuration, consult the main documentation or reach out to the DevOps team.