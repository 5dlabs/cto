Implement subtask 9005: Harden MinIO for production with versioning and lifecycle policies

## Objective
Configure MinIO for production use — either scale a dedicated tenant to 4+ nodes with erasure coding or configure the shared MinIO instance with appropriate replication. Enable bucket versioning and set a 365-day lifecycle retention policy.

## Steps
1. Determine the MinIO deployment model (dedicated tenant vs shared — see decision point).
2. For dedicated tenant: update MinIO Tenant CR to 4+ servers with erasure coding enabled. For shared: verify the production Hermes bucket exists with appropriate access policies.
3. Enable bucket versioning on the `hermes-production` bucket via MinIO client (`mc versioning enable`).
4. Create a lifecycle policy: 365-day retention for all objects, transition older objects to infrequent access tier if available.
5. Set resource requests/limits for MinIO pods if using dedicated tenant.
6. Verify the MinIO endpoint in `hermes-infra-endpoints` ConfigMap points to the production MinIO service.

## Validation
Verify bucket versioning is enabled: `mc versioning info hermes-production/hermes`. Upload an object, delete it, and verify the previous version is recoverable via `mc ls --versions`. Verify lifecycle policy is applied: `mc ilm ls hermes-production/hermes` shows the 365-day retention rule.