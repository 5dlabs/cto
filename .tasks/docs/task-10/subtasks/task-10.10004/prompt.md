Implement subtask 10004: Implement automated secret rotation for PostgreSQL credentials

## Objective
Configure automated 90-day secret rotation for PostgreSQL credentials using CloudNative-PG's built-in mechanism or external-secrets-operator, with zero-downtime rolling restart of dependent services.

## Steps
1. Choose rotation mechanism (see decision point): CNPG built-in or external-secrets-operator.
2. For CNPG built-in: configure the `Cluster` CR's `managed.roles` section to define application user credentials, or use the auto-generated `<cluster>-app` secret with periodic rotation.
3. Create a CronJob or use external-secrets-operator `ExternalSecret` with `refreshInterval: 2160h` (90 days) to trigger credential rotation.
4. Implement zero-downtime rotation workflow:
   a. New secret is generated/updated.
   b. Trigger rolling restart of the backend Deployment: use `kubectl rollout restart` or a Reloader controller that watches Secret changes.
   c. Old credentials remain valid during the rolling update window.
   d. After all pods are updated, optionally revoke old credentials.
5. Add annotations to the Secret: `rotation-schedule: 90d`, `last-rotated: <timestamp>`.
6. Test the full rotation cycle in staging before production.

## Validation
Trigger a manual rotation of PostgreSQL credentials. Verify: (1) New secret is created/updated in Kubernetes. (2) Backend pods are rolling-restarted within 60 seconds. (3) No 500 errors during the rotation window (continuous health check during rotation). (4) After rotation, backend can query the database successfully. (5) Secret annotations show updated `last-rotated` timestamp.