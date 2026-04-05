Implement subtask 1011: Validate all infrastructure resources are healthy and connected

## Objective
Run a comprehensive validation suite to confirm every provisioned resource is ready, accessible, and correctly configured — CNPG cluster healthy, schema isolation enforced, Valkey responding, ConfigMap populated, ExternalSecrets synced, NetworkPolicies active.

## Steps
1. CNPG validation:
   - `kubectl get cluster sigma1-postgres -n sigma1 -o jsonpath='{.status.phase}'` returns healthy.
   - Connect as each per-service user and verify:
     a. Can create/drop a table in their own schema.
     b. Cannot access tables in other schemas (test SELECT, INSERT, CREATE TABLE).
     c. All users can INSERT into audit schema but cannot SELECT/DELETE.
   - Verify search_path for each user is correctly scoped.
2. Valkey validation:
   - `redis-cli -h sigma1-valkey.sigma1.svc.cluster.local PING` returns PONG.
   - `redis-cli INFO server` shows Valkey 7.2.
3. R2 validation:
   - Using stored credentials, perform `aws s3 ls s3://sigma1-assets/ --endpoint-url $R2_ENDPOINT`.
4. ExternalSecrets validation:
   - All 7 ExternalSecret CRs show `SecretSynced` condition.
   - Corresponding Secrets exist with expected keys.
5. ConfigMap validation:
   - `sigma1-infra-endpoints` has all expected keys with non-empty values.
   - `sigma1-rbac-roles` has valid JSON in `roles.json`.
6. NetworkPolicy validation:
   - Cross-namespace connectivity to sigma1-postgres is blocked.
   - Intra-namespace connectivity to sigma1-postgres and sigma1-valkey works.
7. ServiceMonitor validation:
   - Prometheus targets show sigma1 endpoints.
8. Document any issues found and confirm all-green status.

## Validation
A validation script (shell or kubectl-based) runs all checks listed above and outputs PASS/FAIL for each. All checks must PASS. The script is idempotent and can be re-run. Specific checks: 6 users x 2 permission tests (own schema access, cross-schema denied) = 12 permission tests all pass. All 7 ExternalSecrets synced. ConfigMap has 10+ keys. NetworkPolicy blocks 1 cross-namespace test. Valkey PING returns PONG.