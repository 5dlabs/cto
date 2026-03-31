Implement subtask 10005: Implement automated secret rotation for Redis credentials

## Objective
Configure automated 90-day secret rotation for Redis credentials with zero-downtime rolling restart of dependent services.

## Steps
1. Create a rotation mechanism for Redis credentials, consistent with the approach chosen for PostgreSQL (external-secrets-operator or CronJob-based).
2. If using external-secrets-operator: create an `ExternalSecret` that generates new Redis credentials and updates the Kubernetes Secret.
3. If using CronJob: create a CronJob that runs `redis-cli CONFIG SET requirepass <new-password>`, updates the Kubernetes Secret, and triggers a rolling restart of backend pods.
4. Implement the same zero-downtime pattern: update secret → rolling restart → verify connectivity.
5. Add annotations: `rotation-schedule: 90d`, `last-rotated: <timestamp>`.
6. Handle Sentinel password updates if Redis Sentinel mode requires separate auth.

## Validation
Trigger a manual rotation of Redis credentials. Verify: (1) New secret is updated in Kubernetes. (2) Backend pods are restarted and can connect to Redis. (3) No errors during the rotation window. (4) `redis-cli AUTH <new-password>` succeeds and `redis-cli AUTH <old-password>` fails (after full rotation).