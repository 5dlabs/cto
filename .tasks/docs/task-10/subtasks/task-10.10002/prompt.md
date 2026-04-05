Implement subtask 10002: Automate secret rotation for database credentials

## Objective
Implement automated rotation of PostgreSQL and Redis credentials using the External Secrets Operator or equivalent, ensuring services pick up new credentials without downtime.

## Steps
1. Install/configure the External Secrets Operator (external-secrets.io) if not already present, or use the chosen secret management solution per dp-12.
2. Create ExternalSecret resources for PostgreSQL credentials (username, password, connection string) that sync from the external secret store.
3. Configure rotation policy with an appropriate interval (e.g., every 30 days).
4. For PostgreSQL, implement a rotation lambda/CronJob that: creates a new password, updates the user in PostgreSQL, updates the external secret store.
5. Ensure Deployments reference secrets via `envFrom` or `env[].valueFrom.secretKeyRef` so new pods pick up rotated values.
6. Test that a rolling restart after rotation causes zero downtime.

## Validation
Trigger a manual secret rotation. Verify the new credential is propagated to the Kubernetes Secret within the configured sync interval. Restart a service pod and verify it connects successfully with the new credential. Verify the old credential is invalidated.