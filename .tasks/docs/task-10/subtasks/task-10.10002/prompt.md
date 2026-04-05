Implement subtask 10002: Implement automated secret rotation for database credentials

## Objective
Set up automated rotation for PostgreSQL database credentials (application user passwords, replication credentials) so that secrets are periodically rotated without causing application downtime.

## Steps
1. Determine the secret rotation mechanism (External Secrets Operator, operator-native rotation, or CronJob-based — per dp-15 decision).
2. For PostgreSQL:
   a. Configure the PostgreSQL operator to support credential rotation (if supported natively).
   b. If not native, create a CronJob that: generates a new password, updates the PostgreSQL user password via SQL, updates the Kubernetes Secret, and triggers a rolling restart of dependent application pods.
3. Set rotation interval (e.g., every 30 days).
4. Ensure the rotation process is atomic — old credentials remain valid until all consumers have picked up the new ones.
5. Store rotation history/metadata as annotations on the Secret or in a ConfigMap for auditability.
6. Test the rotation flow end-to-end in a staging/dev environment before applying to production.

## Validation
Trigger a manual rotation and verify: the PostgreSQL password is changed, the Kubernetes Secret is updated, application pods pick up the new credentials (via restart or dynamic reload), and the application continues serving requests without errors. Verify old credentials no longer work after rotation completes.