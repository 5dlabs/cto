Implement subtask 3002: Configure Redis HA with Sentinel mode

## Objective
Switch the Redis deployment from standalone to Sentinel mode for production, with 3 nodes and automatic failover, and update the REDIS_URL in the ConfigMap to a sentinel-aware connection string.

## Steps
1. In `values-prod.yaml`, update the Bitnami Redis subchart values:
   - `architecture: replication` (enables master + replicas + sentinel).
   - `sentinel.enabled: true`
   - `replica.replicaCount: 2` (1 master + 2 replicas = 3 nodes total).
   - `sentinel.quorum: 2`
   - `master.persistence.size: 1Gi`
   - `replica.persistence.size: 1Gi`
2. Update `infra/notifycore/templates/configmap-endpoints.yaml` with conditional REDIS_URL:
   - Dev: `redis://:<password>@notifycore-redis-master.notifycore.svc:6379`
   - Prod: sentinel-aware URL. Note: the Rust `redis` crate supports sentinel via `redis+sentinel://` scheme or by configuring sentinel nodes directly. Set REDIS_URL to sentinel format or add separate `REDIS_SENTINEL_NODES` and `REDIS_MASTER_NAME` env vars.
3. Add a PodDisruptionBudget for Redis pods (maxUnavailable: 1).
4. Ensure `values-dev.yaml` retains standalone architecture.

## Validation
`helm template` with values-prod.yaml renders Redis with sentinel enabled, 3 total Redis pods. ConfigMap REDIS_URL for prod uses sentinel-aware connection string. `values-dev.yaml` rendering still shows standalone Redis. PDB resource for Redis is present in prod rendering.