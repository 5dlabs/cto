Implement subtask 1003: Deploy single-replica Redis instance via Bitnami Helm chart

## Objective
Deploy a single-replica Redis instance named `notifycore-redis` using the Bitnami Helm chart as a subchart dependency, with `requirepass` stored in Secret `notifycore-redis-auth`.

## Steps
1. Add Bitnami Redis as a Helm subchart dependency in `infra/notifycore/Chart.yaml` (repository: https://charts.bitnami.com/bitnami, name: redis, version: ~18.x).
2. In `values-dev.yaml`, configure redis subchart values:
   - architecture: standalone (single replica)
   - auth.enabled: true
   - auth.existingSecret: `notifycore-redis-auth` (or let the chart create it)
   - master.persistence.size: 512Mi
   - replica.replicaCount: 0
3. Create `infra/notifycore/templates/redis-auth-secret.yaml` with a generated password stored under key `redis-password` (or use Helm random function).
4. Run `helm dependency update infra/notifycore/` to fetch the subchart.
5. Ensure the Redis master service is named `notifycore-redis-master` for DNS consistency.

## Validation
Redis pod `notifycore-redis-master-0` is Running/Ready. `kubectl get secret notifycore-redis-auth -n notifycore` exists with `redis-password` key. `redis-cli -h notifycore-redis-master.notifycore.svc -a <password> PING` returns PONG.