Implement subtask 1007: Deploy Redis via Helm Subchart per Namespace

## Objective
Deploy a single-replica Redis instance using bitnami/redis as a Helm subchart dependency in each namespace, with persistence disabled for staging and enabled for production. Assumes the Redis deployment method decision point has been resolved.

## Steps
Step-by-step:
1. Add `bitnami/redis` as a dependency in `Chart.yaml` with condition `redis.enabled` (default true) and alias `redis`.
2. Configure in `values.yaml` defaults:
   - `redis.architecture: standalone`
   - `redis.auth.enabled: true`
   - `redis.auth.password`: parameterized (must be set per-environment)
   - `redis.master.persistence.enabled: false` (default for staging)
   - `redis.replica.replicaCount: 0`
   - `redis.nameOverride: hermes-redis`
3. In `values-staging.yaml`: `redis.master.persistence.enabled: false`
4. In `values-production.yaml`: `redis.master.persistence.enabled: true`, `redis.master.persistence.size: 5Gi`
5. Run `helm dependency update charts/hermes-infra` to fetch the subchart.
6. Connection string pattern: `redis://:{{ password }}@hermes-redis-master.{{ .Values.namespace }}.svc:6379` — document this for the Secret/ConfigMap subtasks.
7. Apply standard labels via subchart `commonLabels` configuration.
8. Verify: `helm template --debug` renders Redis resources correctly for both environments.

## Validation
`kubectl get pods -n hermes-staging -l app.kubernetes.io/name=hermes-redis` shows 1 running pod. A test pod runs `redis-cli -h hermes-redis-master -a <password> PING` and receives `PONG`. In staging, no PVC exists for Redis (`kubectl get pvc -n hermes-staging -l app.kubernetes.io/name=hermes-redis` returns nothing). In production, a PVC exists with 5Gi size.