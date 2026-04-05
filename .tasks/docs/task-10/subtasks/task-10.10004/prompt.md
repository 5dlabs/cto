Implement subtask 10004: Configure secret rotation policies for ExternalSecrets, CNPG, and R2

## Objective
Set up ExternalSecrets refresh intervals, CNPG database password rotation schedule, R2 API key rotation documentation, and create the operational runbook ConfigMap.

## Steps
1. Update all ExternalSecret CRs to set `refreshInterval: 1h` to poll the external secret store hourly.
2. Configure CNPG cluster CR to enable scheduled password rotation every 90 days (if CNPG supports it natively) or create a CronJob that:
   - Generates a new password
   - Updates the CNPG user password via SQL
   - Updates the corresponding Kubernetes Secret
   - Triggers rolling restarts of services that use the database
3. For R2 API key rotation: document the manual process (regenerate key in Cloudflare dashboard → update ExternalSecret source → ExternalSecret syncs within 1 hour).
4. Create ConfigMap `sigma1-ops-runbooks` containing markdown documentation for:
   - JWT token manual rotation procedure
   - Database password rotation procedure
   - R2 API key rotation procedure
   - Emergency secret revocation steps
5. Add annotations to all secret-bearing resources indicating rotation schedule and owner.

## Validation
Verify all ExternalSecret CRs have `refreshInterval: 1h`. Verify CNPG password rotation mechanism works by triggering it manually and confirming services reconnect successfully. Verify `sigma1-ops-runbooks` ConfigMap exists and contains all 4 runbook sections. Verify ExternalSecret status shows last refresh within expected interval.