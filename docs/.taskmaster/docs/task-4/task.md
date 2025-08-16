# Task 4: PR Validation DAG WorkflowTemplate (Clippy → QA) with Compliance Gates

## Overview

This task implements a comprehensive PR validation workflow as an Argo DAG that enforces code quality standards through a structured Clippy → QA pipeline with strict compliance gates. The workflow ensures zero warnings, proper formatting, and provides Kubernetes-based proof of QA verification before allowing PR approval.

## Architecture

The PR validation system consists of four interconnected components:

1. **Clippy Format Step**: Automated code formatting and linting
2. **QA Testing Step**: Quality assurance verification with Kubernetes proof
3. **Compliance Verification**: Independent enforcement of zero-warning policy
4. **Kubernetes Proof Validation**: Evidence collection and verification

## Key Features

### Multi-Language Support
- **Rust**: cargo fmt, clippy with pedantic warnings
- **Node.js**: prettier, eslint with zero warnings
- **Go**: gofmt, golangci-lint with strict rules
- **Python**: black, ruff with comprehensive checking
- **Shell**: shellcheck for script validation

### Compliance Enforcement
- **Zero Warnings Policy**: All linting tools must report zero warnings
- **Format Compliance**: Code must pass format checks without changes
- **Independent Validation**: Separate verification step beyond agent execution
- **Tool Version Tracking**: Logs exact versions of all tools used

### Kubernetes Evidence Collection
- **Cluster State Capture**: kubectl inventory of deployed resources
- **Log Collection**: Complete logs from test execution
- **Evidence Validation**: Structured validation of collected proof
- **Artifact Storage**: Permanent storage of verification evidence

## Implementation Details

### DAG Structure

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: pr-validation
spec:
  entrypoint: dag
  templates:
    - name: dag
      dag:
        tasks:
          - name: clippy-format
            templateRef:
              name: coderun-template
              template: coderun-main
            arguments:
              parameters:
                - name: github-app
                  value: clippy
          - name: qa-testing
            dependencies: [clippy-format]
            templateRef:
              name: coderun-template
              template: coderun-main
            arguments:
              parameters:
                - name: github-app
                  value: qa
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

### Language Detection and Validation

The compliance verification step automatically detects the project language and applies appropriate tools:

**Detection Logic**:
```bash
# Language detection priority
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
```

**Rust Validation**:
```bash
rustup component add rustfmt clippy || true
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic
```

**Node.js Validation**:
```bash
npm ci
npx prettier --check .
npx eslint . --max-warnings=0
```

**Go Validation**:
```bash
test -z "$(gofmt -l .)" || { echo 'gofmt issues'; exit 1; }
golangci-lint run --timeout 5m
```

**Python Validation**:
```bash
pip install -U pip black ruff
black --check .
ruff check --no-cache
```

### QA Kubernetes Proof Schema

The QA step must generate evidence following this structure:

**Directory Layout**:
```
/artifacts/qa/proof/
├── summary.json          # Required: structured summary
├── logs/
│   ├── pod1-container.log # kubectl logs output
│   └── pod2-container.log
├── k8s/
│   ├── pods.yaml         # kubectl get pods -o yaml
│   ├── services.yaml     # kubectl get services -o yaml
│   └── events.yaml       # kubectl get events -o yaml
└── evidence/
    ├── test-results.xml  # Test execution results
    └── coverage.json     # Coverage reports
```

**summary.json Schema**:
```json
{
  "cluster": "dev-cluster-01",
  "namespace": "pr-123-test",
  "testCases": [
    {
      "name": "integration-api-tests",
      "status": "passed",
      "details": "All 45 API tests passed successfully"
    },
    {
      "name": "e2e-ui-tests", 
      "status": "passed",
      "details": "UI automation completed without errors"
    }
  ],
  "evidence": [
    {
      "path": "logs/api-server.log",
      "type": "log"
    },
    {
      "path": "k8s/pods.yaml",
      "type": "kubernetes-resource"
    },
    {
      "path": "evidence/test-results.xml",
      "type": "test-results"
    }
  ]
}
```

### Compliance Summary Schema

The verify-compliance step generates a structured summary:

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

### Verification Templates

**verify-compliance Template**:
```yaml
- name: verify-compliance
  container:
    image: ghcr.io/myorg/ci-multilang:latest
    command: ["/bin/bash"]
    args: ["-c", "/scripts/verify-compliance.sh"]
    env:
      - name: ARTIFACTS_DIR
        value: /artifacts/compliance
    volumeMounts:
      - name: workspace
        mountPath: /workspace
      - name: artifacts
        mountPath: /artifacts
  outputs:
    artifacts:
      - name: compliance-summary
        path: /artifacts/compliance/summary.json
```

**verify-k8s-proof Template**:
```yaml  
- name: verify-k8s-proof
  container:
    image: ghcr.io/myorg/kubectl-jq:1.30
    command: ["/bin/bash"]
    args: ["-c", "/scripts/verify-k8s-proof.sh"]
    env:
      - name: QA_ARTIFACTS_DIR
        value: /artifacts/qa/proof
    volumeMounts:
      - name: artifacts
        mountPath: /artifacts
  outputs:
    artifacts:
      - name: verification-result
        path: /artifacts/qa/proof/verification.json
```

## Configuration Override

Projects can customize validation behavior using `.pr-validation.yml`:

```yaml
language: rust
fmtCommand: "cargo fmt --all -- --check"
lintCommand: "cargo clippy --workspace --all-targets -- -D warnings"
workingDir: "."
timeout: 300
allowedWarnings: 0
skipSteps: []
customTools:
  - name: "custom-linter"
    command: "./scripts/custom-lint.sh"
    required: true
```

## Error Handling and Recovery

### Failure Modes

1. **Compliance Failures**: When linting tools report warnings or format issues
2. **Missing QA Proof**: When QA step doesn't generate required evidence
3. **Kubernetes Verification Failures**: When cluster state validation fails
4. **Tool Execution Errors**: When compliance tools crash or are unavailable

### Recovery Strategies

- **Retry Logic**: Transient tool failures trigger automatic retries
- **Graceful Degradation**: Missing optional evidence doesn't fail the entire workflow
- **Clear Error Messages**: Each failure mode provides specific remediation guidance
- **Tool Version Fallbacks**: Automatic fallback to alternative tool versions

### Error Message Examples

```bash
# Compliance failure
"Compliance check failed: 3 clippy warnings found. All warnings must be resolved."

# Missing QA proof
"QA verification failed: summary.json not found at /artifacts/qa/proof/summary.json"

# Kubernetes evidence missing
"Evidence validation failed: Referenced log file logs/api-server.log does not exist"
```

## Monitoring and Observability

### Key Metrics

- **Validation Success Rate**: Percentage of PR validations that pass all gates
- **Compliance Gate Effectiveness**: Rate of issues caught by compliance verification
- **QA Evidence Quality**: Completeness score of generated Kubernetes proof
- **Tool Execution Time**: Performance metrics for each validation tool

### Alerting

- **Compliance Drift**: Alert when warning counts trend upward
- **QA Proof Failures**: Alert when evidence generation fails repeatedly
- **Tool Version Issues**: Alert when validation tools are outdated
- **Cluster Access Problems**: Alert when Kubernetes evidence collection fails

### Dashboards

- **PR Validation Overview**: Success rates, failure categories, processing time
- **Language-Specific Metrics**: Tool performance by programming language
- **Evidence Quality Trends**: QA proof completeness over time
- **Compliance Effectiveness**: Warning detection and resolution patterns

## Integration Points

### GitHub Integration

- **Status Checks**: Workflow status reflected in PR status checks
- **Review Comments**: Failures generate descriptive PR comments
- **Commit Status**: Each gate reports status to specific commit SHA
- **Branch Protection**: Integration with GitHub branch protection rules

### Artifact Storage

- **S3/MinIO**: Long-term storage of compliance summaries and QA evidence
- **Retention Policies**: Automatic cleanup of old artifacts
- **Access Controls**: Role-based access to validation artifacts
- **Search and Discovery**: Indexing of validation results for analysis

### Notification Systems

- **Slack Integration**: Real-time notifications for validation failures
- **Email Alerts**: Summary reports for compliance trends
- **Dashboard Links**: Direct links to detailed validation results
- **Escalation Paths**: Automatic escalation for repeated failures

## Security Considerations

### Access Controls

- **RBAC Permissions**: Minimal permissions for validation pods
- **Namespace Isolation**: Separate namespaces for different validation contexts
- **Secret Management**: Secure handling of tool credentials and API keys
- **Network Policies**: Restricted network access for validation pods

### Evidence Integrity

- **Artifact Signing**: Cryptographic signing of validation artifacts
- **Tamper Detection**: Checksums and integrity verification
- **Audit Trails**: Complete audit log of validation activities
- **Compliance Records**: Immutable records for regulatory compliance

### Tool Security

- **Container Scanning**: Regular security scans of validation tool containers
- **Dependency Management**: Pinned versions of all validation tools
- **Vulnerability Monitoring**: Automated detection of tool vulnerabilities
- **Supply Chain Security**: Verification of tool provenance and integrity

## Performance Optimization

### Parallelization

- **Independent Gates**: Compliance and QA verification run in parallel
- **Tool Parallelism**: Multiple validation tools execute concurrently
- **Resource Allocation**: Optimized CPU and memory allocation per tool
- **Caching Strategies**: Intelligent caching of tool outputs and dependencies

### Resource Management

- **Node Affinity**: Scheduling validation workloads on appropriate nodes
- **Resource Limits**: Prevent resource exhaustion from validation tools
- **Spot Instance Usage**: Cost optimization through spot instance utilization
- **Auto-scaling**: Dynamic scaling based on validation queue depth

### Optimization Techniques

- **Incremental Validation**: Only validate changed files when possible
- **Tool Result Caching**: Cache validation results for unchanged code
- **Parallel Tool Execution**: Run multiple validation tools simultaneously
- **Early Termination**: Stop validation on first critical failure

## Troubleshooting Guide

### Common Issues

**Compliance verification fails with "command not found":**
```bash
# Check container image includes all required tools
kubectl run debug --rm -it --image=ghcr.io/myorg/ci-multilang:latest -- which cargo

# Verify tool versions
kubectl run debug --rm -it --image=ghcr.io/myorg/ci-multilang:latest -- cargo --version
```

**QA proof validation fails:**
```bash
# Check QA artifacts directory
kubectl exec workflow-pod -c qa-testing -- ls -la /artifacts/qa/proof/

# Validate summary.json structure
kubectl exec workflow-pod -c qa-testing -- jq . /artifacts/qa/proof/summary.json
```

**Kubernetes evidence collection fails:**
```bash
# Check kubectl access and permissions
kubectl auth can-i get pods --as=system:serviceaccount:workflows:qa-sa

# Test cluster connectivity
kubectl exec workflow-pod -c verify-k8s-proof -- kubectl cluster-info
```

### Debug Commands

```bash
# Monitor workflow execution
argo watch pr-validation-xyz123

# Check specific step logs
argo logs pr-validation-xyz123 -c verify-compliance

# Inspect generated artifacts
kubectl exec workflow-pod -- find /artifacts -type f -exec ls -la {} \;

# Validate tool execution
kubectl exec workflow-pod -- /scripts/verify-compliance.sh --dry-run
```

## Future Enhancements

### Planned Features

1. **Advanced Language Support**: Additional programming languages and frameworks
2. **Custom Rule Sets**: Project-specific linting rules and compliance policies  
3. **Performance Benchmarking**: Automated performance regression detection
4. **Security Scanning**: Integration with security vulnerability scanners
5. **Dependency Analysis**: License compliance and vulnerability checking

### Integration Opportunities

1. **IDE Integration**: Real-time validation feedback in development environments
2. **CI/CD Pipeline**: Integration with existing continuous integration systems
3. **Code Review Tools**: Enhanced code review with validation insights
4. **Metrics Platform**: Advanced analytics and reporting capabilities

## Dependencies

### External Dependencies

- Argo Workflows 3.4+
- Kubernetes cluster with RBAC enabled
- Container registry access for validation tool images
- GitHub App integration (from Task 2)
- Artifact storage system (S3, MinIO, etc.)

### Internal Dependencies

- coderun-template (from Task 3)
- GitHub App token generator (from Task 2) 
- MCP requirements configuration
- System prompt ConfigMaps

### Tool Dependencies

- Language-specific validation tools (rustfmt, clippy, prettier, eslint, etc.)
- kubectl and jq for Kubernetes operations
- yq for YAML processing
- Container runtime and orchestration platform

## References

- [Argo Workflows DAG](https://argoproj.github.io/argo-workflows/walk-through/dag/)
- [Kubernetes RBAC](https://kubernetes.io/docs/reference/access-authn-authz/rbac/)
- [Rust Clippy Lints](https://rust-lang.github.io/rust-clippy/stable/index.html)
- [ESLint Rules](https://eslint.org/docs/rules/)
- [Black Code Formatter](https://black.readthedocs.io/)
- [golangci-lint](https://golangci-lint.run/)