# Subtask 1.3: Deploy Redis/Valkey Instance

## Parent Task
Task 1

## Agent
redis-deployer

## Parallelizable
Yes

## Description
Deploy Redis/Valkey cluster for caching and session storage with persistence.

## Details
- Deploy Valkey or Redis cluster with appropriate topology
- Configure persistence (RDB + AOF)
- Set up Redis Cluster for sharding
- Implement memory management policies
- Configure network policies for cache access
- Set up monitoring for cache hit rates

## Deliverables
- `redis-cluster.yaml` - Redis/Valkey cluster CR
- `redis-configmap.yaml` - Runtime configuration
- `redis-secrets.yaml` - Credentials

## Acceptance Criteria
- [ ] Redis/Valkey cluster is Running
- [ ] Cache reads/writes succeed
- [ ] Persistence survives restart
- [ ] Cluster sharding works correctly

## Testing Strategy
- Connect with redis-cli and verify keys
- Test SET/GET operations
- Verify persistence after restart
- Test cluster failover
