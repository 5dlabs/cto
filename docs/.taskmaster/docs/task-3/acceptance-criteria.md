# Acceptance Criteria: Simplified CodeRun API with Auto-Detection

## Overview

This document defines comprehensive acceptance criteria for the simplified CodeRun/DocsRun API implementation. The system must reduce required parameters to 1-2 while maintaining full functionality through intelligent auto-detection.

## Functional Requirements

### Template Structure and API

**API-001: Simplified Parameter Interface**
- [ ] coderun-template requires only `github-app` parameter
- [ ] Optional parameters: `event` (default "{}"), `taskRef` (default "")
- [ ] docsrun-template has identical parameter interface
- [ ] press-play accepts `backlog`, `parallelism`, and `event` parameters
- [ ] Invalid github-app values are rejected with clear error messages

**API-002: GitHub App Parameter Validation**
- [ ] Accepts exactly these values: rex, clippy, qa, triage, security
- [ ] Rejects any other values with descriptive error message
- [ ] Case-sensitive validation (lowercase required)
- [ ] Error message lists all valid options

**API-003: Template Registration**
- [ ] Templates are properly registered in Argo Workflows
- [ ] Templates appear in `argo template list` output
- [ ] Templates pass `argo template lint` validation
- [ ] Templates can be submitted via `argo submit --from workflowtemplate/NAME`

### Event Payload Auto-Detection

**AD-001: Pull Request Event Processing**
- [ ] Extracts repository owner from `.repository.owner.login`
- [ ] Extracts repository name from `.repository.name`
- [ ] Extracts PR number from `.pull_request.number`
- [ ] Extracts head ref from `.pull_request.head.ref`
- [ ] Extracts head SHA from `.pull_request.head.sha`
- [ ] Constructs full repository name as "owner/repo"

**AD-002: Issue Event Processing**
- [ ] Extracts repository information from `.repository` object
- [ ] Extracts issue number from `.issue.number`
- [ ] Handles issue events without PR context
- [ ] Distinguishes between issue and PR events correctly

**AD-003: Workflow Run Event Processing**
- [ ] Extracts branch from `.workflow_run.head_branch`
- [ ] Extracts head SHA from `.workflow_run.head_sha`
- [ ] Extracts workflow run ID from `.workflow_run.id`
- [ ] Handles workflow run completion events

**AD-004: Push Event Processing**
- [ ] Extracts branch reference from `.ref` field
- [ ] Extracts commit SHA from `.after` field
- [ ] Handles push to main/master branches
- [ ] Processes forced pushes correctly

**AD-005: Organization Events**
- [ ] Falls back to `.organization.login` when `.repository.owner.login` not available
- [ ] Handles organization-level events correctly
- [ ] Maintains compatibility with repository-level events

### Fallback Logic

**FB-001: Git-Based Fallbacks**
- [ ] Uses `git rev-parse --abbrev-ref HEAD` when ref not in event
- [ ] Only attempts git commands when `/work/src/.git` exists
- [ ] Handles git command failures gracefully (no crash)
- [ ] Continues with defaults when git fallback fails

**FB-002: Default Value Assignment**
- [ ] Sets `ref` to "main" when no other source available
- [ ] Constructs `repoFullName` from owner/repo when not in event
- [ ] Provides empty strings for optional fields when unavailable
- [ ] Never leaves required fields undefined

**FB-003: Error Conditions**
- [ ] Fails with exit code 78 when repository info completely unavailable
- [ ] Provides clear error message listing what's missing
- [ ] Includes troubleshooting hints in error messages
- [ ] Logs attempted fallback methods before failing

### Context Processing

**CP-001: JSON Parsing**
- [ ] Handles well-formed JSON event payloads correctly
- [ ] Gracefully handles malformed JSON (uses defaults)
- [ ] Processes empty event payload "{}" without errors
- [ ] Handles missing fields in JSON structure

**CP-002: Output Parameter Generation**
- [ ] Generates all required output parameters: repoFullName, owner, repo, ref, prNumber, issueNumber, workflowRunId, sha, isPR
- [ ] Parameters are available to downstream templates
- [ ] Parameter values match event payload or fallback sources
- [ ] isPR parameter correctly set (1 for PRs, 0 otherwise)

**CP-003: Logging and Debug Output**
- [ ] Logs resolved parameters for debugging (no secrets)
- [ ] Shows which values came from event vs fallback vs default
- [ ] Provides correlation between input event and resolved context
- [ ] No sensitive information in debug output

### System Prompt Resolution

**SP-001: Prompt File Discovery**
- [ ] Checks `/etc/agents/${GITHUB_APP}_system-prompt.md` first
- [ ] Falls back to `/etc/agents/default_system-prompt.md` if app-specific not found
- [ ] Fails clearly when neither file exists
- [ ] Returns full file path as output parameter

**SP-002: ConfigMap Integration**
- [ ] Successfully mounts controller-agents ConfigMap at `/etc/agents`
- [ ] ConfigMap is mounted read-only
- [ ] All GitHub App prompt files are accessible
- [ ] Default prompt file exists as fallback

**SP-003: Validation Logic**
- [ ] Validates file existence before proceeding
- [ ] Provides clear error when prompt files missing
- [ ] Error message includes searched paths
- [ ] Fails fast without attempting CR creation

### Custom Resource Creation

**CR-001: CodeRun CR Generation**
- [ ] Creates valid CodeRun CR with correct API version
- [ ] All auto-detected parameters populate CR spec correctly
- [ ] System prompt path is set from validation step
- [ ] GitHub App name matches input parameter
- [ ] Workspace path is set to "/work/src"
- [ ] MCP requirements file path is set to "/work/requirements.yaml"
- [ ] Token file path is set to "/var/run/github/token"

**CR-002: DocsRun CR Generation**
- [ ] Creates valid DocsRun CR with correct structure
- [ ] User prompt path is set to "docs/task.md"
- [ ] Docs path is set to "docs/"
- [ ] Docs action is set to "build-preview"
- [ ] All other fields match CodeRun pattern

**CR-003: Resource Template Usage**
- [ ] Uses Kubernetes resource template action: create
- [ ] CR is created in the correct namespace
- [ ] Generated names use appropriate prefix (coderun-, docsrun-)
- [ ] CRs are discoverable via kubectl commands

### Volume and Secret Integration

**VS-001: Workspace Volume Setup**
- [ ] EmptyDir volume mounted at `/work/src`
- [ ] Volume is writable by workflow containers
- [ ] Workspace persists across workflow steps
- [ ] Git operations work in workspace directory

**VS-002: ConfigMap Mounts**
- [ ] controller-agents ConfigMap mounted at `/etc/agents`
- [ ] mcp-requirements ConfigMap content available at `/work/requirements.yaml`
- [ ] Both mounts are read-only
- [ ] Files are accessible to all workflow containers

**VS-003: GitHub Token Integration**
- [ ] GitHub App token is available at `/var/run/github/token`
- [ ] Token file has correct permissions (0600)
- [ ] Token corresponds to specified GitHub App
- [ ] Token is valid and usable for GitHub API calls

**VS-004: Security Context**
- [ ] InitContainers run as non-root user (65532)
- [ ] Read-only root filesystem where possible
- [ ] No sensitive data in environment variables
- [ ] Secrets mounted securely via volume projection

### Press-Play Orchestrator

**PP-001: Batch Processing**
- [ ] Accepts JSON array of backlog items
- [ ] Each item has githubApp and taskRef fields
- [ ] Creates one workflow per backlog item
- [ ] Parallelism limit is respected

**PP-002: Concurrency Control**
- [ ] spec.parallelism set from workflow parameter
- [ ] Maximum concurrent workflows limited appropriately
- [ ] Workflow queue management works correctly
- [ ] No resource exhaustion under load

**PP-003: Event Propagation**
- [ ] Original event parameter passed to child workflows
- [ ] Event payload preserved across batch items
- [ ] Child workflows receive correct GitHub App parameter
- [ ] TaskRef parameter correctly passed through

## Integration Requirements

### Argo Events Integration

**AE-001: Event Payload Compatibility**
- [ ] Works with pull_request webhook payloads
- [ ] Works with issues webhook payloads
- [ ] Works with workflow_run webhook payloads
- [ ] Works with push webhook payloads
- [ ] Handles Argo Events parameter passing format

**AE-002: Sensor Integration**
- [ ] Templates can be triggered from Argo Events Sensors
- [ ] Event payload correctly passed via workflow parameters
- [ ] Template references work in Sensor trigger configurations
- [ ] No data loss or corruption in event-to-workflow pipeline

### GitHub App Token Integration

**GT-001: Token Generator Integration**
- [ ] InitContainer pattern works with token generator from Task 2
- [ ] GitHub App secrets are correctly resolved by parameter
- [ ] Token generation succeeds for all supported GitHub Apps
- [ ] Generated tokens are valid for GitHub API operations

**GT-002: Secret Mounting**
- [ ] Secrets mounted based on github-app parameter value
- [ ] Different GitHub Apps use their respective secrets
- [ ] No cross-contamination between GitHub App credentials
- [ ] Error handling when secrets are missing

### Kubernetes Resource Integration

**KR-001: RBAC Permissions**
- [ ] Workflow service account can create CodeRun CRs
- [ ] Workflow service account can create DocsRun CRs
- [ ] Workflow service account can read required ConfigMaps
- [ ] Workflow service account can read GitHub App secrets
- [ ] No excessive permissions granted

**KR-002: Namespace Isolation**
- [ ] All resources created in appropriate namespace
- [ ] Cross-namespace access only where necessary
- [ ] Network policies allow required communication
- [ ] Resource quotas respected

## Error Handling Requirements

### Input Validation

**IV-001: Parameter Validation**
- [ ] Invalid github-app parameter causes immediate failure
- [ ] Error message lists all valid github-app values
- [ ] Malformed JSON event payload handled gracefully
- [ ] Missing required fields detected and reported

**IV-002: Event Processing Errors**
- [ ] JSON parsing errors don't crash the workflow
- [ ] Missing event fields handled with appropriate defaults
- [ ] Oversized event payloads handled gracefully
- [ ] Event structure validation provides helpful error messages

### Resource Errors

**RE-001: ConfigMap Errors**
- [ ] Missing controller-agents ConfigMap causes clear error
- [ ] Missing system prompt files cause clear error with paths
- [ ] Missing mcp-requirements ConfigMap handled gracefully
- [ ] ConfigMap mount failures detected and reported

**RE-002: Secret Errors**
- [ ] Missing GitHub App secrets cause clear error messages
- [ ] Invalid secret format detected and reported
- [ ] Secret access permission errors handled appropriately
- [ ] Clear guidance provided for secret troubleshooting

**RE-003: CR Creation Errors**
- [ ] Invalid CR manifests cause workflow failure with details
- [ ] CRD availability checked before CR creation
- [ ] Resource quota exceeded errors handled gracefully
- [ ] Namespace permission errors provide clear guidance

### Recovery Mechanisms

**RM-001: Retry Logic**
- [ ] Transient failures trigger appropriate retries
- [ ] Exponential backoff implemented for retry delays
- [ ] Maximum retry attempts configured reasonably
- [ ] Permanent failures don't trigger infinite retries

**RM-002: Graceful Degradation**
- [ ] System works with partial event data
- [ ] Default values allow workflow to proceed when possible
- [ ] Non-critical errors don't block essential functionality
- [ ] Clear logging distinguishes between critical and non-critical errors

## Performance Requirements

### Response Time

**RT-001: Template Execution Speed**
- [ ] Context processing completes within 10 seconds
- [ ] System prompt validation completes within 5 seconds
- [ ] CR creation completes within 5 seconds
- [ ] End-to-end template execution under 30 seconds

**RT-002: Auto-Detection Performance**
- [ ] JSON parsing completes within 1 second for typical payloads
- [ ] Event payloads up to 1MB processed efficiently
- [ ] No performance degradation with complex event structures
- [ ] Memory usage remains stable during processing

### Scalability

**SC-001: Concurrent Execution**
- [ ] Supports 50+ concurrent template executions
- [ ] No resource contention between concurrent workflows
- [ ] Shared volumes don't cause performance bottlenecks
- [ ] ConfigMap access scales with concurrent users

**SC-002: Batch Processing**
- [ ] Press-play handles 100+ backlog items efficiently
- [ ] Parallelism controls prevent resource exhaustion
- [ ] Large batch jobs complete within reasonable time
- [ ] Memory usage linear with batch size, not exponential

### Resource Usage

**RU-001: Container Resources**
- [ ] Context processing uses <100MB memory
- [ ] CPU usage remains under 100m during normal operation
- [ ] Container startup time under 5 seconds
- [ ] Resource limits prevent runaway processes

**RU-002: Storage Efficiency**
- [ ] Workspace volumes sized appropriately for typical repos
- [ ] No excessive disk usage from temporary files
- [ ] Log output size manageable for long-running workflows
- [ ] Cleanup prevents storage leaks

## Security Requirements

### Input Sanitization

**IS-001: Event Payload Security**
- [ ] No shell injection possible through event fields
- [ ] Path traversal attempts blocked in file operations
- [ ] Special characters in repository names handled safely
- [ ] No code execution possible through event payload

**IS-002: Parameter Validation Security**
- [ ] GitHub App parameter validated against strict whitelist
- [ ] TaskRef parameter sanitized to prevent injection
- [ ] No environment variable injection through parameters
- [ ] Command line arguments properly escaped

### Secret Protection

**SP-001: Credential Security**
- [ ] No secrets logged in plain text
- [ ] No secrets exposed in environment variable dumps
- [ ] GitHub tokens protected with appropriate file permissions
- [ ] Private keys never written to persistent storage

**SP-002: Access Control**
- [ ] Minimal RBAC permissions for workflow execution
- [ ] Cross-namespace access restricted appropriately
- [ ] GitHub App secrets isolated per application
- [ ] No privilege escalation possible through workflow execution

### Network Security

**NS-001: Communication Security**
- [ ] All external API calls use HTTPS/TLS
- [ ] Certificate validation enabled for external connections
- [ ] No sensitive data transmitted in URLs or query parameters
- [ ] Internal communication secured appropriately

## Testing Requirements

### Unit Testing

**UT-001: Event Processing Tests**
- [ ] Test all supported GitHub event types
- [ ] Test malformed JSON handling
- [ ] Test missing field scenarios
- [ ] Test oversized payload handling
- [ ] Verify all auto-detected parameters for each event type

**UT-002: Fallback Logic Tests**
- [ ] Test git-based fallback when event data missing
- [ ] Test default value assignment
- [ ] Test error conditions when all fallbacks fail
- [ ] Test git command failure handling

**UT-003: Validation Tests**
- [ ] Test all valid github-app parameter values
- [ ] Test invalid github-app parameter handling
- [ ] Test system prompt file discovery
- [ ] Test ConfigMap mounting scenarios

### Integration Testing

**IT-001: End-to-End Workflow Tests**
- [ ] Submit coderun-template with various event payloads
- [ ] Verify CodeRun CRs created with correct specifications
- [ ] Test docsrun-template with documentation events
- [ ] Verify DocsRun CRs created with appropriate configurations

**IT-002: Press-Play Integration Tests**
- [ ] Test batch processing with multiple GitHub Apps
- [ ] Verify parallelism controls work correctly
- [ ] Test large backlog processing
- [ ] Verify error handling in batch scenarios

**IT-003: GitHub Integration Tests**
- [ ] Test with real GitHub webhook payloads
- [ ] Verify GitHub App token generation and usage
- [ ] Test repository access with generated tokens
- [ ] Validate GitHub API operations work correctly

### Load Testing

**LT-001: Concurrent Execution Tests**
- [ ] Test 50+ concurrent template executions
- [ ] Measure resource usage under load
- [ ] Verify no deadlocks or race conditions
- [ ] Test recovery after resource exhaustion

**LT-002: Large Data Tests**
- [ ] Test with 1MB event payloads
- [ ] Test with 1000+ item backlogs in press-play
- [ ] Verify memory usage remains stable
- [ ] Test performance with large repository contexts

## Operational Requirements

### Monitoring

**MON-001: Metrics Collection**
- [ ] Template execution success/failure rates tracked
- [ ] Auto-detection accuracy metrics available
- [ ] Performance metrics (execution time) collected
- [ ] Error rate metrics by error type

**MON-002: Alerting**
- [ ] Alert on template execution failures
- [ ] Alert on auto-detection failures
- [ ] Alert on ConfigMap or Secret access issues
- [ ] Alert on performance degradation

**MON-003: Logging**
- [ ] Structured logging with correlation IDs
- [ ] Debug logs show auto-detection decisions
- [ ] Error logs provide actionable troubleshooting information
- [ ] No sensitive information in logs

### Documentation

**DOC-001: Usage Documentation**
- [ ] Clear examples for each template usage pattern
- [ ] Documentation of all supported event types
- [ ] Troubleshooting guide for common issues
- [ ] Integration examples with Argo Events

**DOC-002: Operational Documentation**
- [ ] Deployment instructions for templates
- [ ] Configuration requirements documented
- [ ] Monitoring and alerting setup guide
- [ ] Disaster recovery procedures

## Validation Procedures

### Pre-Deployment Testing

1. **Template Validation**
   - [ ] Run `argo template lint` on all templates
   - [ ] Submit templates with test parameters
   - [ ] Verify CR creation in test namespace
   - [ ] Test error conditions and recovery

2. **Integration Validation**
   - [ ] Test with sample GitHub webhook payloads
   - [ ] Verify end-to-end event processing
   - [ ] Test GitHub App integration
   - [ ] Validate press-play batch processing

3. **Security Validation**
   - [ ] Review RBAC configurations
   - [ ] Test input sanitization
   - [ ] Verify secret handling
   - [ ] Conduct basic penetration testing

### Production Readiness Checklist

1. **Infrastructure Requirements**
   - [ ] Argo Workflows 3.4+ installed and configured
   - [ ] External Secrets Operator deployed (Task 2)
   - [ ] GitHub App token generator available (Task 2)
   - [ ] Required ConfigMaps created and populated

2. **Configuration Validation**
   - [ ] All GitHub App secrets properly configured
   - [ ] System prompt files available in controller-agents ConfigMap
   - [ ] MCP requirements file available in mcp-requirements ConfigMap
   - [ ] RBAC permissions configured correctly

3. **Performance Validation**
   - [ ] Load testing completed successfully
   - [ ] Resource usage within acceptable limits
   - [ ] Response times meet requirements
   - [ ] Scalability targets achieved

4. **Security Validation**
   - [ ] Security review completed and approved
   - [ ] No secrets exposed in logs or environment
   - [ ] Network security policies applied
   - [ ] Access controls validated

## Success Criteria Summary

The implementation is considered complete and production-ready when:

1. **API Simplification**: Templates require only github-app parameter for basic usage
2. **Auto-Detection**: All repository and event context automatically detected
3. **Resource Creation**: CodeRun and DocsRun CRs created correctly
4. **Error Handling**: Comprehensive error handling with clear messages
5. **Integration**: Seamless integration with Argo Events and GitHub Apps
6. **Performance**: Meets all performance and scalability requirements
7. **Security**: All security requirements satisfied
8. **Testing**: Comprehensive test suite passes all scenarios
9. **Documentation**: Complete operational and usage documentation
10. **Monitoring**: Observability and alerting fully implemented

Each criterion must be explicitly verified and documented before the system can be considered production-ready.