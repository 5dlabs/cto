# Acceptance Criteria: PR Validation DAG with Compliance Gates

## Overview

This document defines comprehensive acceptance criteria for the PR validation DAG system. The implementation must enforce zero-warning code quality through independent compliance gates and provide tamper-proof Kubernetes evidence of testing.

## Functional Requirements

### DAG Structure and Dependencies

**DS-001: Workflow Template Structure**
- [ ] pr-validation WorkflowTemplate exists with entrypoint "dag"
- [ ] DAG contains exactly 5 tasks: clippy-format, qa-testing, verify-compliance, verify-k8s-proof, gates-passed
- [ ] clippy-format has no dependencies (runs first)
- [ ] qa-testing depends only on clippy-format
- [ ] verify-compliance depends only on clippy-format
- [ ] verify-k8s-proof depends only on qa-testing
- [ ] gates-passed depends on both verify-compliance AND verify-k8s-proof

**DS-002: Parallel Execution Validation**
- [ ] verify-compliance and qa-testing execute in parallel after clippy-format completes
- [ ] verify-k8s-proof waits for qa-testing but not verify-compliance
- [ ] gates-passed only executes if both verification gates succeed
- [ ] Failed verification gates prevent gates-passed from executing

**DS-003: Template References**
- [ ] clippy-format uses templateRef to coderun-template with github-app=clippy
- [ ] qa-testing uses templateRef to coderun-template with github-app=qa
- [ ] verify-compliance uses local template (not templateRef)
- [ ] verify-k8s-proof uses local template (not templateRef)
- [ ] All templateRefs specify correct template name (coderun-main)

### Multi-Language Compliance Verification

**ML-001: Language Detection**
- [ ] Detects Rust projects via Cargo.toml presence
- [ ] Detects Node.js projects via package.json presence
- [ ] Detects Go projects via go.mod presence
- [ ] Detects Python projects via pyproject.toml or requirements.txt presence
- [ ] Falls back to shell validation when no language files detected
- [ ] Respects .pr-validation.yml language override when present

**ML-002: Rust Validation**
- [ ] Executes `rustup component add rustfmt clippy` safely
- [ ] Runs `cargo fmt --all -- --check` and fails on formatting issues
- [ ] Runs `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
- [ ] Fails when any clippy warnings or errors are present
- [ ] Reports exact warning count and type in error messages
- [ ] Records rustfmt and clippy versions in compliance summary

**ML-003: Node.js Validation**
- [ ] Executes `npm ci` to install dependencies
- [ ] Runs `npx prettier --check .` and fails on formatting issues
- [ ] Runs `npx eslint . --max-warnings=0` with zero-warning enforcement
- [ ] Fails when any ESLint warnings or errors are present
- [ ] Records prettier and eslint versions in compliance summary

**ML-004: Go Validation**
- [ ] Runs `gofmt -l .` and fails when files need formatting
- [ ] Executes `golangci-lint run --timeout 5m` with comprehensive checking
- [ ] Fails when any linting issues are detected
- [ ] Records gofmt and golangci-lint versions in compliance summary

**ML-005: Python Validation**
- [ ] Installs latest versions of black and ruff
- [ ] Runs `black --check .` and fails on formatting issues
- [ ] Runs `ruff check --no-cache` with comprehensive rule set
- [ ] Fails when any formatting or linting issues detected
- [ ] Records black and ruff versions in compliance summary

**ML-006: Shell Validation**
- [ ] Runs `shellcheck` on all shell scripts (.sh, .bash files)
- [ ] Fails when any shellcheck issues are detected
- [ ] Provides clear error messages for shell script issues
- [ ] Records shellcheck version in compliance summary

### Configuration Override System

**CO-001: Configuration File Support**
- [ ] Reads .pr-validation.yml from repository root when present
- [ ] Parses YAML configuration safely with error handling
- [ ] Supports language override to force specific validation type
- [ ] Supports custom fmtCommand and lintCommand overrides
- [ ] Supports workingDir specification for monorepos

**CO-002: Configuration Validation**
- [ ] Validates configuration file schema before use
- [ ] Rejects configurations that could enable code execution
- [ ] Falls back to auto-detection when configuration is invalid
- [ ] Logs configuration parsing errors clearly
- [ ] Provides example configuration in error messages

**CO-003: Security Constraints**
- [ ] Configuration commands are validated against allowed patterns
- [ ] No arbitrary command execution through configuration
- [ ] Path traversal attempts in workingDir are blocked
- [ ] Configuration parsing is safe from YAML bombs and injection

### QA Evidence Collection and Validation

**QA-001: Evidence Directory Structure**
- [ ] QA step creates /artifacts/qa/proof/ directory structure
- [ ] Creates subdirectories: logs/, k8s/, evidence/
- [ ] summary.json file is present in proof/ root directory
- [ ] All referenced evidence files exist at specified paths

**QA-002: Summary.json Schema Validation**
- [ ] Contains required fields: cluster, namespace, testCases, evidence
- [ ] testCases is array with name, status, details fields
- [ ] evidence is array with path, type fields
- [ ] All testCases have status in [passed, failed, skipped]
- [ ] No testCases with failed status when validation should pass

**QA-003: Kubernetes Resource Collection**
- [ ] kubectl get pods -o yaml saved to k8s/pods.yaml
- [ ] kubectl get services -o yaml saved to k8s/services.yaml
- [ ] kubectl get events --sort-by=.lastTimestamp -o yaml saved to k8s/events.yaml
- [ ] All kubectl commands handle namespace correctly
- [ ] Resource files are valid YAML and contain expected structure

**QA-004: Log Collection**
- [ ] kubectl logs collected for all pods in test namespace
- [ ] Logs saved with naming pattern {pod-name}.log
- [ ] Multi-container pod logs collected with --all-containers=true
- [ ] Log collection failures don't fail the overall evidence collection
- [ ] Empty log files handled gracefully

**QA-005: Evidence File Validation**
- [ ] All evidence files referenced in summary.json exist
- [ ] Evidence file paths are relative to /artifacts/qa/proof/
- [ ] Evidence files contain expected content (not empty unless expected)
- [ ] File types match declared types in evidence array
- [ ] Evidence collection generates verification.json with validation results

### Compliance Summary Generation

**CS-001: Summary File Structure**
- [ ] Generates /artifacts/compliance/summary.json file
- [ ] Contains language, tools, fmtStatus, lintStatus, warningsCount, timestamp fields
- [ ] tools array contains name and version for each validation tool
- [ ] fmtStatus and lintStatus are either "passed" or "failed"
- [ ] warningsCount is integer (must be 0 for passed status)
- [ ] timestamp is valid RFC3339 format

**CS-002: Tool Version Recording**
- [ ] Records exact versions of all validation tools used
- [ ] Version information matches actual tool versions executed
- [ ] Handles cases where version detection fails gracefully
- [ ] Tool version format is consistent across languages
- [ ] Version recording doesn't fail validation on version detection errors

**CS-003: Status Accuracy**
- [ ] fmtStatus reflects actual formatting validation results
- [ ] lintStatus reflects actual linting validation results
- [ ] warningsCount exactly matches number of warnings found
- [ ] Failed validation correctly sets status to "failed"
- [ ] Summary generation never misrepresents validation results

### Verification Gate Implementation

**VG-001: Compliance Verification Logic**
- [ ] Reads compliance summary from /artifacts/compliance/summary.json
- [ ] Fails when summary.json is missing
- [ ] Fails when warningsCount > 0
- [ ] Fails when fmtStatus != "passed"
- [ ] Fails when lintStatus != "passed"
- [ ] Provides clear error messages for each failure type

**VG-002: K8s Proof Verification Logic**  
- [ ] Reads QA summary from /artifacts/qa/proof/summary.json
- [ ] Validates all testCases have status "passed"
- [ ] Verifies all evidence files referenced in summary.json exist
- [ ] Fails when any required files are missing
- [ ] Generates verification.json with detailed validation results

**VG-003: Gate Dependencies**
- [ ] verify-compliance cannot run before clippy-format completes
- [ ] verify-k8s-proof cannot run before qa-testing completes  
- [ ] gates-passed cannot run unless both verification gates succeed
- [ ] Failed gates prevent downstream execution
- [ ] Gate execution is properly logged and traceable

## Container and Infrastructure Requirements

### Container Image Specifications

**CI-001: Multi-Language Container (ci-multilang)**
- [ ] Contains bash, git, jq, yq for basic operations
- [ ] Includes Rust: rustup, rustfmt, clippy
- [ ] Includes Node.js: node, npm, npx, prettier, eslint
- [ ] Includes Go: go compiler, gofmt, golangci-lint
- [ ] Includes Python: python3, pip, black, ruff
- [ ] Includes shellcheck for shell script validation
- [ ] Runs as non-root user (65532:65532)
- [ ] Uses minimal base image (alpine or distroless)
- [ ] Image size under 500MB compressed

**CI-002: Kubernetes Validation Container (kubectl-jq)**
- [ ] Contains kubectl client version 1.30+
- [ ] Includes jq for JSON processing
- [ ] Includes bash for scripting
- [ ] Configured with appropriate cluster access
- [ ] Runs as non-root user
- [ ] Image size under 100MB compressed

### RBAC and Security

**RS-001: Service Account Configuration**
- [ ] Dedicated service account for pr-validation workflows
- [ ] Minimal RBAC permissions for required operations
- [ ] Can read pods, services, events in test namespaces
- [ ] Can read pod logs in test namespaces
- [ ] Cannot modify cluster resources
- [ ] Cannot access secrets outside workflow namespace

**RS-002: Container Security**
- [ ] All containers run as non-root users
- [ ] Read-only root filesystem where possible
- [ ] Dropped capabilities except required ones
- [ ] No privileged containers required
- [ ] Security contexts properly configured

**RS-003: Network Security**
- [ ] Network policies restrict unnecessary pod communication
- [ ] No outbound internet access except for package managers
- [ ] Inter-pod communication limited to required services
- [ ] TLS used for all external API communications

### Volume and Artifact Management

**VA-001: Artifact Storage**
- [ ] Shared /artifacts volume accessible to all workflow steps
- [ ] Proper subdirectory structure (/artifacts/compliance/, /artifacts/qa/proof/)
- [ ] Volume persists across workflow steps
- [ ] Appropriate volume size limits configured
- [ ] Artifact cleanup after workflow completion

**VA-002: Workspace Management**
- [ ] Source code workspace shared between steps
- [ ] Git operations work correctly in workspace
- [ ] File permissions allow all required operations
- [ ] Workspace isolation between concurrent workflows

## Error Handling and Recovery

### Error Message Requirements

**EM-001: Compliance Error Messages**
- [ ] Clear indication of which tools failed
- [ ] Exact count of warnings/errors found
- [ ] Remediation suggestions for each failure type
- [ ] No sensitive information exposed in error messages
- [ ] Error messages include context about validation rules

**EM-002: Evidence Error Messages**
- [ ] Clear indication of missing evidence files
- [ ] Schema validation errors with field details
- [ ] Kubernetes access error messages with remediation
- [ ] Evidence collection failures don't mask validation results

**EM-003: Configuration Error Messages**
- [ ] YAML parsing errors with line numbers
- [ ] Configuration validation failures with examples
- [ ] Fallback behavior clearly documented in messages
- [ ] Security constraint violations clearly explained

### Failure Recovery

**FR-001: Transient Failure Handling**
- [ ] Network timeouts retry with exponential backoff
- [ ] Tool installation failures retry appropriately  
- [ ] Kubernetes API failures retry with backoff
- [ ] Non-critical operations (log collection) don't fail workflow

**FR-002: Permanent Failure Handling**
- [ ] Invalid configuration fails fast with clear messages
- [ ] Missing required tools fail fast
- [ ] Authentication/authorization failures provide clear guidance
- [ ] Resource constraint failures include resource requirements

## Performance Requirements

### Execution Time

**ET-001: Step Performance**
- [ ] Language detection completes in <5 seconds
- [ ] Compliance validation completes in <10 minutes for typical projects
- [ ] Evidence collection completes in <5 minutes
- [ ] Evidence validation completes in <2 minutes
- [ ] Total workflow under 30 minutes including agent steps

**ET-002: Parallel Execution**
- [ ] verify-compliance and qa-testing execute simultaneously
- [ ] No unnecessary blocking between independent steps
- [ ] Resource usage optimized for parallel execution
- [ ] Concurrency limits prevent resource exhaustion

### Resource Usage

**RU-001: CPU and Memory**
- [ ] Compliance validation uses <2 CPU cores peak
- [ ] Memory usage stays under 4GB per container
- [ ] Resource requests set appropriately for scheduling
- [ ] No memory leaks in long-running validations

**RU-002: Storage**
- [ ] Artifact storage under 100MB per validation
- [ ] No excessive temporary file usage
- [ ] Workspace cleanup after validation
- [ ] Log rotation for long-running operations

## Integration Requirements

### GitHub Integration

**GI-001: Status Reporting**
- [ ] Workflow status reflected in GitHub PR status checks
- [ ] Individual gate status reported separately
- [ ] Commit status updates include links to detailed results
- [ ] Status descriptions include failure summaries

**GI-002: Comment Generation**
- [ ] Failed validations generate PR comments with details
- [ ] Comments include remediation guidance
- [ ] Multiple validation failures consolidated in single comment
- [ ] Comments include links to full validation logs

### Artifact Integration

**AI-001: Storage Integration**
- [ ] Compliance summaries stored in configured artifact repository
- [ ] QA evidence uploaded to long-term storage
- [ ] Artifact URLs included in workflow outputs
- [ ] Retention policies applied to stored artifacts

**AI-002: Monitoring Integration**
- [ ] Metrics emitted for validation success/failure rates
- [ ] Performance metrics for each validation step
- [ ] Error rates tracked by failure category
- [ ] Dashboard integration for validation trends

## Testing Requirements

### Unit Testing

**UT-001: Language Validation Testing**
- [ ] Test each language with clean code (should pass)
- [ ] Test each language with warnings (should fail)
- [ ] Test each language with formatting issues (should fail)
- [ ] Test configuration override functionality
- [ ] Test language detection accuracy

**UT-002: Evidence Validation Testing**
- [ ] Test with complete evidence (should pass)
- [ ] Test with missing summary.json (should fail)
- [ ] Test with invalid summary.json schema (should fail)
- [ ] Test with missing evidence files (should fail)
- [ ] Test with failed test cases (should fail)

### Integration Testing

**IT-001: End-to-End Workflow Testing**
- [ ] Submit complete workflow with passing validation
- [ ] Submit workflow with compliance failures
- [ ] Submit workflow with QA evidence failures
- [ ] Submit workflow with both gates failing
- [ ] Verify gates-passed only runs when both gates succeed

**IT-002: Real Project Testing**
- [ ] Test with actual Rust project (clean and with warnings)
- [ ] Test with actual Node.js project (clean and with issues)
- [ ] Test with actual Go project (formatted and unformatted)
- [ ] Test with actual Python project (black formatted and unformatted)
- [ ] Test with monorepo using workingDir configuration

### Performance Testing

**PT-001: Load Testing**
- [ ] Multiple concurrent validations execute successfully
- [ ] Resource usage remains stable under load
- [ ] No deadlocks or race conditions under concurrent execution
- [ ] Validation queue management works properly

**PT-002: Large Project Testing**
- [ ] Validation works with projects >1000 files
- [ ] Memory usage scales appropriately with project size
- [ ] Timeout handling for very large projects
- [ ] Incremental validation optimization where possible

## Security Testing

### Security Validation

**SV-001: Container Security Testing**
- [ ] Containers cannot escalate privileges
- [ ] No sensitive data in container images
- [ ] Container vulnerability scans pass
- [ ] Runtime security policies enforced

**SV-002: Configuration Security Testing**
- [ ] Malicious .pr-validation.yml files rejected
- [ ] Command injection attempts blocked
- [ ] Path traversal attempts blocked
- [ ] YAML bomb attacks handled gracefully

**SV-003: Evidence Integrity Testing**
- [ ] Evidence files cannot be tampered with
- [ ] Validation process detects evidence modification
- [ ] Checksums verify evidence file integrity
- [ ] Audit trail captures all validation activities

## Operational Requirements

### Monitoring

**MON-001: Metrics Collection**
- [ ] pr_validation_total counter with labels (language, status)
- [ ] pr_validation_duration_seconds histogram with percentiles
- [ ] compliance_warnings_total counter by language and tool
- [ ] qa_evidence_completeness_ratio gauge
- [ ] gate_failure_total counter by gate type

**MON-002: Alerting**
- [ ] Alert on high validation failure rates
- [ ] Alert on compliance drift (increasing warnings)
- [ ] Alert on evidence collection failures
- [ ] Alert on gate processing failures
- [ ] Alert on performance degradation

### Logging

**LOG-001: Structured Logging**
- [ ] All logs in JSON format with correlation IDs
- [ ] Log levels appropriately set (DEBUG, INFO, WARN, ERROR)
- [ ] No sensitive information in logs
- [ ] Searchable by workflow, PR, repository
- [ ] Retention policies configured

**LOG-002: Audit Logging**
- [ ] All validation decisions logged with rationale
- [ ] Tool versions and configurations logged
- [ ] Evidence collection activities logged
- [ ] Gate execution logged with outcomes

## Documentation Requirements

### User Documentation

**UD-001: Configuration Guide**
- [ ] Complete .pr-validation.yml schema documentation
- [ ] Examples for each supported language
- [ ] Common configuration patterns documented
- [ ] Troubleshooting guide for configuration issues

**UD-002: Integration Guide**
- [ ] GitHub App setup instructions
- [ ] Argo Events integration examples
- [ ] Branch protection rule configuration
- [ ] Troubleshooting guide for integration issues

### Operational Documentation

**OD-001: Deployment Guide**
- [ ] Complete deployment manifests
- [ ] RBAC configuration examples
- [ ] Container image build instructions
- [ ] Dependency installation guide

**OD-002: Troubleshooting Guide**
- [ ] Common failure scenarios with solutions
- [ ] Performance optimization guidance
- [ ] Debug commands and procedures
- [ ] Escalation procedures for complex issues

## Acceptance Testing Procedures

### Pre-Production Validation

1. **Template Validation**
   - [ ] `argo template lint pr-validation.yaml` passes
   - [ ] Template submits successfully in test environment
   - [ ] DAG structure visualizes correctly in Argo UI

2. **Language Testing**
   - [ ] Create test projects for each supported language
   - [ ] Introduce warnings/formatting issues
   - [ ] Verify validation fails appropriately
   - [ ] Fix issues and verify validation passes

3. **Evidence Testing**
   - [ ] Deploy test application to Kubernetes
   - [ ] Run QA validation with evidence collection
   - [ ] Verify all evidence files collected correctly
   - [ ] Test evidence validation with missing files

4. **Integration Testing**
   - [ ] Connect to actual GitHub repository
   - [ ] Trigger validation from PR events
   - [ ] Verify status checks and comments
   - [ ] Test with multiple concurrent PRs

### Production Readiness Checklist

1. **Security Review**
   - [ ] RBAC permissions audited and approved
   - [ ] Container security scan results reviewed
   - [ ] Configuration security validated
   - [ ] Evidence integrity mechanisms tested

2. **Performance Validation**
   - [ ] Load testing completed successfully
   - [ ] Resource usage profiled and optimized
   - [ ] Scalability limits documented
   - [ ] Monitoring and alerting configured

3. **Operational Readiness**
   - [ ] Runbooks created and tested
   - [ ] Support team trained on troubleshooting
   - [ ] Backup and recovery procedures documented
   - [ ] Incident response procedures established

## Success Criteria Summary

The implementation is considered complete and production-ready when:

1. **DAG Execution**: Complete pipeline executes with proper dependencies and parallel gates
2. **Multi-Language Support**: All five languages validate correctly with zero-warning enforcement  
3. **Evidence System**: QA evidence collection and validation works for Kubernetes environments
4. **Gate Enforcement**: Independent compliance and proof verification prevent unqualified code
5. **Error Handling**: Clear, actionable error messages for all failure scenarios
6. **Security**: All security requirements met including RBAC, container security, evidence integrity
7. **Performance**: Meets all performance targets under normal and load conditions
8. **Integration**: Seamless integration with GitHub, Argo Events, and artifact storage
9. **Testing**: Comprehensive test suite validates all functionality and edge cases
10. **Operations**: Complete monitoring, logging, documentation, and support procedures

Each criterion must be explicitly verified and documented with evidence before the system can be considered production-ready.