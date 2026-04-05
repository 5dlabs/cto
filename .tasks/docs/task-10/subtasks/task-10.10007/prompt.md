Implement subtask 10007: Penetration test RBAC and secret access controls

## Objective
Perform security validation by attempting unauthorized access patterns against RBAC policies, secrets, and audit logging to verify controls are effective.

## Steps
1. RBAC testing: a) From each service account, attempt to perform actions outside its granted permissions (e.g., backend SA trying to delete pods, list secrets in other namespaces). b) Attempt to escalate privileges by creating new RoleBindings from a non-admin SA. c) Attempt to access the Kubernetes API from within a pod without a mounted ServiceAccount token. 2. Secret access testing: a) Attempt to read secrets from a pod whose SA is not authorized for secret access. b) Verify that secret values are not exposed in pod environment variable listings (prefer volume mounts). c) Attempt to access the external secrets backend directly from an application pod without proper credentials. 3. Audit log verification: a) Perform all the above unauthorized attempts and verify each one generates an audit log entry. b) Verify audit logs cannot be tampered with from within application pods. 4. Document all findings with pass/fail status and remediation actions for any failures.

## Validation
All unauthorized access attempts must be blocked (return 403 or timeout). Every blocked attempt must appear in the audit logs within 60 seconds. No privilege escalation path should be exploitable. Produce a security test report with all test cases, expected results, actual results, and pass/fail status. Zero critical findings must remain unresolved.