Implement subtask 10006: Implement automated secret rotation for MinIO credentials

## Objective
Configure automated 90-day secret rotation for MinIO access keys with zero-downtime rolling restart and verification that artifact read/write continues working.

## Steps
1. Create a rotation mechanism for MinIO access key and secret key.
2. If using external-secrets-operator: create an `ExternalSecret` with rotation.
3. If using CronJob: create a CronJob that:
   a. Creates a new MinIO service account/access key via `mc admin user svcacct add`.
   b. Updates the Kubernetes Secret with new credentials.
   c. Triggers rolling restart of backend pods.
   d. After verification, removes the old access key via `mc admin user svcacct rm`.
4. Ensure the rotation script verifies artifact read/write with the new credentials before revoking old ones.
5. Add annotations: `rotation-schedule: 90d`, `last-rotated: <timestamp>`.

## Validation
Trigger a manual rotation of MinIO credentials. Verify: (1) New credentials are stored in the Kubernetes Secret. (2) Backend pods restart and can upload/download artifacts from MinIO. (3) Old credentials are revoked after successful verification. (4) An existing artifact can be read with the new credentials.