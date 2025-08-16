# Acceptance Criteria: GitHub App Token Generation System

## Overview

This document defines the comprehensive acceptance criteria for the GitHub App token generation system. All criteria must be met for the implementation to be considered complete and production-ready.

## Functional Requirements

### External Secrets Integration

**ES-001: Secret Store Configuration**
- [ ] SecretStore resource is configured for the chosen provider (AWS Secrets Manager, Azure Key Vault, etc.)
- [ ] SecretStore has proper IAM/RBAC permissions to read GitHub App credentials
- [ ] Connection to external secret store is validated and working
- [ ] Network policies allow External Secrets Operator to reach the secret store

**ES-002: ExternalSecret Resources**
- [ ] ExternalSecret exists for each GitHub App: rex, clippy, qa, triage, security
- [ ] Each ExternalSecret has correct remote key paths for appId and privateKey
- [ ] RefreshInterval is set to 1 hour
- [ ] Target secret naming follows pattern: `github-app-{APP_NAME}`

**ES-003: Kubernetes Secret Generation**
- [ ] Kubernetes Secrets are automatically created by ExternalSecrets
- [ ] Each secret contains both `appId` and `privateKey` keys
- [ ] Secret values match the corresponding values in the external secret store
- [ ] Secrets are created in the correct namespace (workflows)

**ES-004: Secret Rotation**
- [ ] Updating a value in the external secret store updates the Kubernetes Secret within refreshInterval
- [ ] Secret rotation does not break running workflows
- [ ] ExternalSecret status shows "Synced" after rotation
- [ ] No downtime occurs during secret rotation

### Token Generator Implementation

**TG-001: Core Functionality**
- [ ] Token generator reads APP_ID and PRIVATE_KEY from environment or mounted volumes
- [ ] Creates valid RS256 JWT with proper claims structure (iss, iat, exp)
- [ ] Successfully exchanges JWT for GitHub installation token
- [ ] Writes token to specified output path with atomic operations
- [ ] Token file has 0600 permissions

**TG-002: JWT Structure Validation**
- [ ] JWT includes required claims: iss (APP_ID), iat (current time - 60s), exp (current time + 540s)
- [ ] JWT is signed using RS256 algorithm with the provided private key
- [ ] JWT can be verified using the corresponding public key
- [ ] JWT expiration is properly validated before use

**TG-003: Installation Discovery**
- [ ] When INSTALLATION_ID is not provided, generator calls GET /app/installations
- [ ] Generator can identify correct installation from available installations
- [ ] Generator handles multiple installations gracefully
- [ ] Error occurs gracefully when no suitable installation is found

**TG-004: GitHub API Integration**
- [ ] Generator calls POST /app/installations/{id}/access_tokens to get installation token
- [ ] Proper headers are sent (Authorization, Accept, User-Agent)
- [ ] Response is parsed correctly to extract token
- [ ] Token expiration time is captured and validated

**TG-005: Error Handling**
- [ ] 401/403 errors fail fast with clear error messages
- [ ] 5xx errors are retried with exponential backoff (5 attempts, 500ms base, max 5s)
- [ ] 429 (rate limiting) errors respect Retry-After header
- [ ] Network errors are handled gracefully with appropriate timeouts
- [ ] Invalid private key format causes immediate failure with clear message

### Containerization

**CT-001: Container Security**
- [ ] Container runs as non-root user (65532:65532)
- [ ] Read-only root filesystem is enabled where possible
- [ ] Container drops all capabilities except necessary ones
- [ ] Base image is minimal (distroless or alpine)
- [ ] No secrets are embedded in the container image

**CT-002: Container Functionality**
- [ ] Container starts successfully with required environment variables
- [ ] Health check endpoint (/healthz) returns 200 when healthy
- [ ] Container exits with appropriate exit codes (0 for success, non-zero for failure)
- [ ] Logs are structured (JSON) and contain no sensitive information
- [ ] Container size is under 50MB compressed

**CT-003: CI/CD Pipeline**
- [ ] Container is built automatically on code changes
- [ ] Image is scanned for vulnerabilities before publishing
- [ ] Image is signed (cosign) and includes SBOM
- [ ] Image is published to the specified registry with proper tags
- [ ] Multi-architecture builds (amd64, arm64) if required

### Workflow Integration

**WI-001: InitContainer Pattern**
- [ ] Token generator works as an initContainer in Argo Workflows
- [ ] InitContainer mounts GitHub App secrets correctly
- [ ] Token is written to shared emptyDir volume at /var/run/github/token
- [ ] Main containers can read the token from the shared volume
- [ ] Token file permissions are correct (0600)

**WI-002: Parameterization**
- [ ] Workflow templates accept github-app parameter to select which app to use
- [ ] Parameter validation ensures only valid app names are accepted
- [ ] Secret mounting is dynamic based on the github-app parameter
- [ ] Error occurs when invalid github-app parameter is provided

**WI-003: Volume Sharing**
- [ ] EmptyDir volume is mounted at /var/run/github in both initContainer and main containers
- [ ] Token file is accessible from main containers
- [ ] File permissions prevent access by other users/groups
- [ ] Volume cleanup occurs after workflow completion

**WI-004: Environment Variables**
- [ ] GITHUB_TOKEN_FILE is set correctly in main containers
- [ ] TOKEN_FILE points to the correct path (/var/run/github/token)
- [ ] No token values are exposed in environment variables
- [ ] Environment variable validation prevents path traversal attacks

### Security Requirements

**SEC-001: Secret Handling**
- [ ] Private keys are never logged at any log level
- [ ] Tokens are never logged or printed to stdout/stderr
- [ ] Memory containing secrets is zeroed after use where possible
- [ ] No secrets appear in environment variable dumps or process lists

**SEC-002: File Permissions**
- [ ] Token files are created with 0600 permissions (owner read/write only)
- [ ] Atomic file operations are used (write to temp file, then rename)
- [ ] Token files are not readable by other users or processes
- [ ] File creation is not vulnerable to symlink attacks

**SEC-003: RBAC Configuration**
- [ ] Service account has minimal required permissions
- [ ] Role only allows reading the specific GitHub App secrets
- [ ] No cluster-wide permissions are granted
- [ ] Service account cannot list or access other secrets

**SEC-004: Network Security**
- [ ] All GitHub API calls use TLS (HTTPS)
- [ ] Certificate validation is properly implemented
- [ ] No sensitive data is transmitted in URLs or query parameters
- [ ] Network policies restrict unnecessary pod-to-pod communication

### Performance Requirements

**PERF-001: Token Generation Speed**
- [ ] Token generation completes within 10 seconds under normal conditions
- [ ] JWT creation occurs within 1 second
- [ ] GitHub API calls complete within 5 seconds (excluding retries)
- [ ] File I/O operations are optimized for minimal latency

**PERF-002: Resource Usage**
- [ ] InitContainer uses minimal CPU and memory resources
- [ ] Memory usage remains stable without leaks
- [ ] Container startup time is under 5 seconds
- [ ] Resource limits are set appropriately

**PERF-003: Scalability**
- [ ] Multiple workflows can run concurrently without token generation conflicts
- [ ] System handles burst usage patterns without failures
- [ ] Rate limiting is respected without causing workflow failures
- [ ] Token caching reduces API calls when appropriate

### Reliability Requirements

**REL-001: Fault Tolerance**
- [ ] System recovers gracefully from temporary GitHub API outages
- [ ] External secret store connectivity issues are handled properly
- [ ] Kubernetes cluster restarts do not affect token generation
- [ ] Partial failures do not leave system in inconsistent state

**REL-002: Retry Logic**
- [ ] Exponential backoff is implemented for transient failures
- [ ] Maximum retry attempts are respected (5 attempts)
- [ ] Jitter is added to prevent thundering herd problems
- [ ] Retry logic does not cause rate limit violations

**REL-003: Monitoring**
- [ ] Token generation success/failure metrics are exposed
- [ ] Secret sync status is monitored
- [ ] JWT expiration warnings are generated
- [ ] API rate limit consumption is tracked

## Test Cases

### Unit Tests

**UT-001: JWT Creation**
- [ ] Valid JWT is created with correct claims
- [ ] JWT signature can be verified with public key
- [ ] Invalid private key format is rejected
- [ ] Clock skew is handled correctly

**UT-002: GitHub API Integration**
- [ ] Installation discovery works with multiple installations
- [ ] Token exchange succeeds with valid JWT
- [ ] Error responses are handled appropriately
- [ ] Rate limiting scenarios are tested

**UT-003: File Operations**
- [ ] Token is written with correct permissions
- [ ] Atomic write operations work correctly
- [ ] File path validation prevents directory traversal
- [ ] Concurrent access scenarios are handled

### Integration Tests

**IT-001: End-to-End Token Flow**
- [ ] External secret is synced to Kubernetes
- [ ] Token generator creates valid installation token
- [ ] Generated token can authenticate with GitHub API
- [ ] Workflow can use token for GitHub operations

**IT-002: Secret Rotation**
- [ ] Backend secret update triggers Kubernetes secret update
- [ ] New tokens use updated credentials
- [ ] Rotation does not break running workflows
- [ ] Status endpoints reflect rotation progress

**IT-003: Error Scenarios**
- [ ] Invalid credentials cause appropriate failures
- [ ] Network errors are handled gracefully
- [ ] GitHub API errors are properly surfaced
- [ ] Cleanup occurs properly after failures

### Security Tests

**ST-001: Secret Exposure**
- [ ] Logs contain no sensitive information
- [ ] Process lists show no secret values
- [ ] Core dumps (if any) contain no secrets
- [ ] Container image contains no embedded secrets

**ST-002: Access Control**
- [ ] Service account cannot access other secrets
- [ ] Token files are not accessible by other users
- [ ] Network access is properly restricted
- [ ] Privilege escalation is not possible

**ST-003: Vulnerability Testing**
- [ ] Container image passes security scans
- [ ] Dependencies have no known high-severity vulnerabilities
- [ ] Input validation prevents injection attacks
- [ ] Error messages do not leak sensitive information

## Operational Requirements

### Monitoring

**MON-001: Metrics**
- [ ] `github_token_generation_total` counter with labels (app, status)
- [ ] `github_token_generation_duration_seconds` histogram
- [ ] `github_jwt_expiration_seconds` gauge
- [ ] `external_secret_sync_total` counter with labels (secret, status)

**MON-002: Alerting**
- [ ] Alert when token generation fails for any GitHub App
- [ ] Alert when secret sync fails
- [ ] Alert when JWT expiration approaches
- [ ] Alert when API rate limits are exceeded

**MON-003: Dashboards**
- [ ] Grafana dashboard shows token generation metrics
- [ ] Dashboard includes error rate and latency percentiles
- [ ] Secret sync status is visualized
- [ ] Historical trends are available

### Documentation

**DOC-001: Operational Documentation**
- [ ] Setup and configuration guide is complete
- [ ] Troubleshooting runbook covers common scenarios
- [ ] Security considerations are documented
- [ ] Monitoring and alerting setup is explained

**DOC-002: Developer Documentation**
- [ ] API documentation is complete and accurate
- [ ] Code contains appropriate comments
- [ ] Architecture decisions are documented
- [ ] Testing procedures are explained

## Validation Procedures

### Pre-Production Checklist

1. **Environment Setup**
   - [ ] External secret store is configured with test credentials
   - [ ] External Secrets Operator is deployed and configured
   - [ ] Test namespace is created with proper RBAC

2. **Deployment Validation**
   - [ ] All Kubernetes resources deploy without errors
   - [ ] ExternalSecrets show "Synced" status
   - [ ] Token generator container starts successfully

3. **Functional Testing**
   - [ ] Run test workflow that generates and uses token
   - [ ] Verify GitHub API authentication works
   - [ ] Test secret rotation scenario
   - [ ] Validate error handling paths

4. **Security Validation**
   - [ ] Review logs for any secret exposure
   - [ ] Verify file permissions and access controls
   - [ ] Test RBAC restrictions
   - [ ] Run security scanner on deployed resources

5. **Performance Validation**
   - [ ] Measure token generation latency
   - [ ] Test concurrent workflow execution
   - [ ] Validate resource usage patterns
   - [ ] Verify rate limiting behavior

### Production Readiness

**PR-001: High Availability**
- [ ] System continues to function with single component failures
- [ ] External secret store outages are handled gracefully
- [ ] Kubernetes node failures do not affect token generation

**PR-002: Disaster Recovery**
- [ ] Backup and restore procedures are documented and tested
- [ ] System can be rebuilt from source control
- [ ] Configuration is stored in version control
- [ ] Recovery time objectives are met

**PR-003: Capacity Planning**
- [ ] Resource requirements are documented
- [ ] Scaling guidelines are provided
- [ ] Rate limits are properly configured
- [ ] Load testing has been performed

## Sign-off Criteria

This implementation is considered complete and ready for production when:

1. All functional requirements are met and tested
2. All security requirements pass validation
3. Performance benchmarks are satisfied
4. All test cases pass consistently
5. Monitoring and alerting are operational
6. Documentation is complete and reviewed
7. Security review is conducted and approved
8. Operational team sign-off is obtained

Each requirement must be explicitly verified and signed off by the appropriate stakeholders before the system can be considered production-ready.