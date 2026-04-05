Implement subtask 10007: Test RBAC enforcement and secret rotation workflows end-to-end

## Objective
Execute comprehensive security tests validating that RBAC policies block unauthorized access, secret rotation works without downtime, and audit logs capture all security-relevant events during these operations.

## Steps
1. **RBAC enforcement tests**:
   a. For each ServiceAccount, attempt operations outside its granted permissions and verify 403 Forbidden.
   b. Attempt to access Secrets from a namespace the SA shouldn't access.
   c. Attempt to escalate privileges (e.g., create a RoleBinding granting cluster-admin).
   d. Verify the default SA in each namespace has no permissions.
2. **Secret rotation tests**:
   a. Trigger database credential rotation and verify application continuity (zero downtime).
   b. Trigger API key rotation and verify external integrations continue working.
   c. Verify old credentials are invalidated after rotation.
   d. Simulate a failed rotation (e.g., provider API unavailable) and verify rollback behavior.
3. **Audit log verification**:
   a. Verify all RBAC test attempts (both allowed and denied) appear in audit logs.
   b. Verify secret access and rotation events are logged.
   c. Verify log timestamps, user identities, and resource details are accurate.
4. Document test results with pass/fail for each scenario.

## Validation
All RBAC denial tests return 403. All secret rotations complete without application errors or downtime (verify via health checks and request success rates during rotation). All test actions appear in audit logs within 60 seconds. Failed rotation simulation triggers appropriate alerts/rollback. Full test report is generated.