Implement subtask 9003: Scale Valkey for production with persistence and PDB

## Objective
Configure Valkey for production use: either sentinel mode (3 nodes) or single-instance with AOF persistence, resource limits, and a PodDisruptionBudget.

## Steps
1. Check Opstree Redis operator documentation for Valkey sentinel support:
   - If sentinel is supported: create a RedisSentinel CR with 3 sentinel nodes and a RedisReplication CR with 1 master + 2 replicas
   - If not supported: update existing Valkey CR for single-instance with persistence
2. For sentinel mode (Option A):
   - Create RedisSentinel CR: `spec.size: 3`
   - Create RedisReplication CR: `spec.size: 3` with sentinel reference
   - Set `spec.redisExporter.enabled: true` for metrics
3. For single persistent mode (Option B):
   - Update existing `sigma1-valkey` CR
   - Enable persistence: add `appendonly yes` and `appendfsync everysec` to Redis config
   - Ensure PVC is configured for data persistence
4. Set resource limits on all Valkey pods:
   - `resources.requests.memory: 512Mi`, `resources.requests.cpu: 250m`
   - `resources.limits.memory: 512Mi`, `resources.limits.cpu: 250m`
5. Configure PodDisruptionBudget:
   - Create a PDB with `maxUnavailable: 1` targeting Valkey pods
6. Verify all pods are running and persistence is active.

## Validation
Verify Valkey pods are running with correct resource limits via `kubectl describe pod`. Write a key to Valkey, delete the pod, wait for restart, and verify the key persists (proving AOF persistence). If sentinel mode, kill the master and verify failover occurs within 30 seconds.