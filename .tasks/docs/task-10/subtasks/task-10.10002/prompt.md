Implement subtask 10002: Implement automated secret rotation for PostgreSQL credentials

## Objective
Configure automated rotation of PostgreSQL user credentials using CloudNative-PG's built-in secret management, ensuring application pods pick up new credentials without downtime.

## Steps
1. Review CloudNative-PG's managed secret lifecycle — CNPG automatically creates secrets for the `app` and `superuser` roles. 2. Configure a CronJob or use an external-secrets operator to trigger periodic credential rotation by updating the CNPG Cluster CR's managed secrets. 3. Alternatively, use CNPG's `managed.roles` feature to define database roles with automatic password rotation on a schedule. 4. Ensure backend service Deployments mount the CNPG-managed secret and use `projected` volumes or `reloader` (stakater/Reloader) to trigger rolling restarts when the secret changes. 5. Install Reloader if not present: add the Helm chart for stakater/Reloader, configure it to watch the PostgreSQL secret. 6. Test: rotate the PostgreSQL secret manually, verify Reloader triggers a rolling restart, verify the backend reconnects with the new credentials.

## Validation
Manually trigger a PostgreSQL credential rotation. Verify Reloader detects the secret change and initiates a rolling restart of dependent Deployments. Confirm zero downtime by running continuous health checks during rotation. Verify the old password no longer works and the new password is in use by checking application database connections.