# Autonomous Agent Prompt: Implement Security and RBAC Controls for Remediation System

## Your Mission
You are tasked with implementing comprehensive security controls, RBAC permissions, and validation systems for the Agent Remediation Loop. This involves creating a multi-layered security framework that protects against unauthorized access, malicious inputs, and resource abuse while maintaining seamless functionality.

## Context
The Agent Remediation Loop handles sensitive operations including GitHub repository access, Kubernetes resource management, and automated code modifications. A robust security framework is essential to:
- Prevent unauthorized access to GitHub repositories and Kubernetes resources
- Protect against injection attacks and malicious content in PR comments
- Implement rate limiting to prevent resource exhaustion and abuse
- Provide comprehensive audit trails for compliance and security monitoring
- Establish least-privilege access controls throughout the system

## Required Actions

### 1. Configure Minimal GitHub Token Permissions
Implement GitHub App permissions with minimal required scope:
- **Repository permissions**: contents:read, issues:write, pull_requests:write, metadata:read, statuses:read, discussions:write
- **Organization permissions**: members:read (if needed for team validation)
- **Token validation system**: Create `GitHubTokenManager` to validate token scopes and permissions
- **Permission testing**: Implement non-destructive validation of operational permissions

### 2. Set Up Kubernetes RBAC for Agent Operations
Create comprehensive RBAC configuration:
- **Service Account**: Create `remediation-controller-sa` in `agent-platform` namespace
- **Role permissions**: Minimal required permissions for ConfigMaps, CodeRuns, Secrets, Events, and Leases
- **ClusterRole permissions**: Limited cluster-wide access for node information and namespace listing
- **RBAC validation**: Implement `RBACValidator` to test permissions programmatically
- **Permission verification**: Use SelfSubjectAccessReview to validate operational permissions

### 3. Implement Comment Validation and Sanitization
Create comprehensive input validation system:
- **Input sanitization**: HTML escape dangerous characters, sanitize shell metacharacters
- **Malicious pattern detection**: Detect script tags, JavaScript URLs, event handlers, template injection
- **Authorization validation**: Check against authorized reviewer allowlist
- **Content validation**: UTF-8 validation, length limits, severity/type validation
- **Audit logging**: Log all validation events for security monitoring

### 4. Add Rate Limiting for API Calls
Implement multi-level rate limiting:
- **Task-based rate limiting**: 100 requests/minute per task with burst capacity
- **GitHub API rate limiting**: Respect GitHub's primary (5000/hour) and secondary (100/min) limits
- **Dynamic rate limiter management**: Per-task rate limiter instances with cleanup routines
- **Context-aware limiting**: Support for waiting with context timeout
- **Rate limit monitoring**: Log rate limit violations and enforcement

### 5. Implement Comprehensive Audit Logging
Create security-focused audit system:
- **Structured audit events**: Timestamp, actor, action, resource, success/failure, metadata
- **Security event logging**: Authentication, authorization, input validation, rate limiting
- **Severity-based logging**: Critical, high, medium, low severity with appropriate log levels
- **Correlation support**: Context-based correlation IDs and request tracking
- **Alert integration**: Support for security alerts and anomaly detection

### 6. Security Manager Integration
Create unified security management:
- **Centralized validation**: Single entry point for security operations validation
- **Security operation structure**: Define SecurityOperation with required context
- **Initialization checks**: System-wide security validation on startup
- **Integration testing**: End-to-end security validation workflows

## Technical Requirements

### Security Framework Components
```go
type SecurityManager struct {
    tokenManager   *GitHubTokenManager
    rbacValidator  *RBACValidator
    inputValidator *InputValidator
    rateLimiter    *RateLimiter
    auditLogger    *AuditLogger
}
```

### RBAC Configuration
```yaml
# Service Account
apiVersion: v1
kind: ServiceAccount
metadata:
  name: remediation-controller-sa
  namespace: agent-platform

# Role with minimal permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: remediation-controller-role
  namespace: agent-platform
rules:
  - apiGroups: [""]
    resources: ["configmaps", "secrets", "events"]
    verbs: ["get", "list", "create", "update", "patch"]
  - apiGroups: ["platform.5dlabs.com"]
    resources: ["coderuns"]
    verbs: ["get", "list", "create", "update", "patch", "delete", "watch"]
```

### Input Validation Framework
- **Structured feedback validation**: Validate StructuredFeedback objects against schema
- **Authorized reviewer checking**: Maintain allowlist of authorized users and bot accounts
- **Content sanitization**: HTML escape and shell metacharacter protection
- **Pattern-based detection**: Regex patterns for common attack vectors

### Rate Limiting Configuration
- **Per-task limiting**: 100 requests/minute with 10 request burst
- **GitHub API limiting**: Respect official GitHub rate limits
- **Cleanup mechanisms**: Automatic cleanup of unused rate limiter instances
- **Context-aware waiting**: Support cancellation and timeout

### Audit Event Structure
```go
type AuditEvent struct {
    Timestamp     time.Time
    EventType     string  // authentication, authorization, input_validation, etc.
    Actor         string
    Action        string
    Resource      string
    Success       bool
    Severity      string  // critical, high, medium, low, info
    Metadata      map[string]interface{}
}
```

## Implementation Checklist

- [ ] Create GitHub token management system with scope validation
- [ ] Configure Kubernetes RBAC with minimal privileges
- [ ] Implement input validation with malicious pattern detection
- [ ] Set up multi-level rate limiting (task-based and GitHub API)
- [ ] Create comprehensive audit logging system
- [ ] Build unified security manager for operation validation
- [ ] Deploy RBAC configurations to cluster
- [ ] Test token permission restrictions
- [ ] Validate input sanitization against attack vectors
- [ ] Test rate limiting enforcement and bypassing attempts
- [ ] Verify audit logging completeness and integrity
- [ ] Perform end-to-end security testing
- [ ] Document security procedures and troubleshooting
- [ ] Set up security monitoring and alerting

## Expected Outputs

1. **Security Go Package**: Complete security framework with all components
2. **RBAC Kubernetes Manifests**: Service accounts, roles, and role bindings
3. **Token Management System**: GitHub App permission validation and testing
4. **Input Validation Framework**: Comprehensive sanitization and validation
5. **Rate Limiting System**: Task-based and API-aware rate limiting
6. **Audit Logging Infrastructure**: Structured security event logging
7. **Security Integration Tests**: Comprehensive test suite for all security components
8. **Documentation**: Security architecture, procedures, and troubleshooting guides

## Success Validation

Your implementation is successful when:
1. **GitHub token permissions** are configured with minimal required scope and validated
2. **Kubernetes RBAC** provides least-privilege access to required resources only
3. **Input validation** successfully blocks all tested injection attacks and malicious content
4. **Rate limiting** prevents resource exhaustion and enforces API limits
5. **Audit logging** captures all security-relevant events with proper severity
6. **Security controls** integrate seamlessly without impacting system functionality
7. **Performance impact** is minimal (<5% overhead for security operations)
8. **Monitoring system** provides proactive security threat detection

## Security Testing Strategy

### Penetration Testing
1. **Input validation testing**: XSS attempts, SQL injection, command injection, template injection
2. **Authentication bypass**: Token manipulation, scope escalation, expired tokens
3. **Authorization testing**: RBAC bypass attempts, privilege escalation
4. **Rate limiting bypass**: Distributed attacks, burst scenarios, concurrent abuse
5. **Audit log tampering**: Log injection, tampering detection, integrity validation

### Security Monitoring
1. **Failed authentication alerts**: Multiple failed token validations
2. **Authorization violations**: RBAC denial patterns, privilege escalation attempts
3. **Input validation failures**: Malicious content detection, unusual pattern frequency
4. **Rate limit breaches**: Sustained high-rate activity, distributed attack patterns
5. **Audit anomalies**: Missing logs, unusual access patterns, correlation failures

### Compliance Testing
1. **Data protection**: No PII in logs, secure token storage, data retention compliance
2. **Access audit trails**: Complete audit coverage, tamper evidence, retention policies
3. **Privilege validation**: Regular access reviews, minimal privilege verification
4. **Security procedures**: Incident response testing, escalation procedures

## Common Security Pitfalls to Avoid

- **Overprivileged access**: Grant only minimum required permissions
- **Insufficient input validation**: Validate all user inputs, not just obvious attack vectors
- **Rate limiting gaps**: Ensure all API endpoints are protected
- **Audit logging gaps**: Log both successful and failed security events
- **Token exposure**: Never log tokens or sensitive authentication data
- **RBAC complexity**: Keep permissions simple and auditable
- **Security through obscurity**: Rely on robust controls, not hidden implementations

## Resources and References

- **GitHub App Permissions**: https://docs.github.com/en/developers/apps/building-github-apps/setting-permissions-for-github-apps
- **Kubernetes RBAC**: https://kubernetes.io/docs/reference/access-authn-authz/rbac/
- **Go Rate Limiting**: golang.org/x/time/rate package documentation
- **Security Best Practices**: OWASP security guidelines and recommendations
- **Audit Logging Standards**: Industry standards for security event logging

## Support and Troubleshooting

If you encounter security issues:
1. **Check audit logs**: Review security event logs for patterns and anomalies
2. **Validate RBAC**: Use kubectl auth can-i to test permissions
3. **Test token scopes**: Verify GitHub token permissions with API calls
4. **Monitor rate limits**: Check rate limiting enforcement and violations
5. **Review input validation**: Test with known attack patterns and edge cases
6. **Verify integration**: Ensure security controls don't break functionality

Begin by implementing the GitHub token management system, then progress through RBAC configuration, input validation, rate limiting, and audit logging. Test each component thoroughly before integration, and maintain security as the top priority throughout implementation.