# Redis Cluster Deployment

This directory contains Kubernetes manifests for deploying a production-ready Redis Cluster with Sentinel for high availability.

## Architecture

- **Redis Cluster**: 6 nodes (3 masters + 3 replicas) with automatic sharding
- **Redis Sentinel**: 3 sentinels for automatic failover monitoring
- **Persistent Storage**: 5Gi per Redis node, 1Gi per Sentinel
- **High Availability**: Pod anti-affinity ensures distribution across nodes

## Components

### Files

- `namespace.yaml`: Creates the databases namespace
- `cluster.yaml`: Redis Cluster StatefulSet with 6 nodes and initialization job
- `sentinel.yaml`: Redis Sentinel StatefulSet with 3 instances
- `service.yaml`: Services for cluster and sentinel access
- `kustomization.yaml`: Kustomize configuration for resource management

### Resource Allocation

**Redis Cluster Nodes:**
- CPU: 250m request, 1000m limit
- Memory: 512Mi request, 1Gi limit
- Storage: 5Gi persistent volume per node

**Redis Sentinel Nodes:**
- CPU: 100m request, 500m limit
- Memory: 128Mi request, 256Mi limit
- Storage: 1Gi persistent volume per node

## Deployment

### Prerequisites

- Kubernetes cluster with at least 6 nodes (recommended for proper anti-affinity)
- Storage class for persistent volumes
- kubectl configured with cluster access

### Deploy

```bash
# Apply all resources using kustomize
kubectl apply -k /workspace/infra/redis/

# Or apply individually
kubectl apply -f namespace.yaml
kubectl apply -f cluster.yaml
kubectl apply -f sentinel.yaml
kubectl apply -f service.yaml
```

### Initialize Cluster

The cluster initialization happens automatically via a Kubernetes Job after pod deployment:

```bash
# Check initialization job status
kubectl get jobs -n databases redis-cluster-init

# View initialization logs
kubectl logs -n databases job/redis-cluster-init
```

### Verify Deployment

```bash
# Check pods
kubectl get pods -n databases -l app=redis-cluster
kubectl get pods -n databases -l app=redis-sentinel

# Check services
kubectl get svc -n databases

# Verify cluster status
kubectl exec -n databases redis-cluster-0 -- redis-cli cluster info
kubectl exec -n databases redis-cluster-0 -- redis-cli cluster nodes

# Check sentinel status
kubectl exec -n databases redis-sentinel-0 -- redis-cli -p 26379 sentinel masters
```

## Usage

### Client Connection

**From within the cluster:**

```bash
# Connect to the cluster service
redis-cli -h redis-cluster.databases.svc.cluster.local -p 6379

# Connect to specific node
redis-cli -h redis-cluster-0.redis-cluster-headless.databases.svc.cluster.local -p 6379
```

**From application code:**

```python
# Python example using redis-py-cluster
from rediscluster import RedisCluster

startup_nodes = [
    {"host": "redis-cluster.databases.svc.cluster.local", "port": "6379"}
]
rc = RedisCluster(startup_nodes=startup_nodes, decode_responses=True)
rc.set("key", "value")
```

### Sentinel Connection

```bash
# Query sentinel for master information
kubectl exec -n databases redis-sentinel-0 -- \
  redis-cli -p 26379 sentinel get-master-addr-by-name redis-cluster
```

## Operations

### Scaling

The cluster is configured with 6 nodes (3 masters, 3 replicas). To scale:

```bash
# Scale the StatefulSet (requires manual resharding)
kubectl scale statefulset redis-cluster -n databases --replicas=9

# After scaling, reshard the cluster
kubectl exec -n databases redis-cluster-0 -- \
  redis-cli --cluster reshard redis-cluster-0.redis-cluster-headless.databases.svc.cluster.local:6379
```

### Monitoring

```bash
# Monitor cluster health
kubectl exec -n databases redis-cluster-0 -- redis-cli cluster info

# Check node distribution
kubectl exec -n databases redis-cluster-0 -- redis-cli cluster nodes

# Monitor sentinel
kubectl exec -n databases redis-sentinel-0 -- redis-cli -p 26379 info sentinel
```

### Backup

```bash
# Trigger manual save
kubectl exec -n databases redis-cluster-0 -- redis-cli bgsave

# Export data from a node
kubectl exec -n databases redis-cluster-0 -- redis-cli --rdb /data/dump.rdb
```

### Troubleshooting

```bash
# Check logs
kubectl logs -n databases redis-cluster-0
kubectl logs -n databases redis-sentinel-0

# Check cluster configuration
kubectl exec -n databases redis-cluster-0 -- cat /data/nodes.conf

# Test connectivity
kubectl exec -n databases redis-cluster-0 -- redis-cli ping

# Check persistent volumes
kubectl get pvc -n databases
```

## High Availability Features

1. **Cluster Mode**: Automatic data sharding across 3 master nodes
2. **Replication**: Each master has 1 replica for data redundancy
3. **Sentinel**: 3 sentinel instances monitor masters and coordinate failover
4. **Pod Anti-Affinity**: Pods distributed across different nodes
5. **Persistent Storage**: Data survives pod restarts
6. **Automatic Failover**: Sentinel promotes replicas when masters fail
7. **Health Checks**: Liveness and readiness probes ensure pod health

## Configuration

### Storage Class

If you need a specific storage class, uncomment and modify in cluster.yaml:

```yaml
volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      storageClassName: fast-ssd  # Specify your storage class
```

### Resource Limits

Adjust resources based on your workload in cluster.yaml and sentinel.yaml:

```yaml
resources:
  requests:
    cpu: 250m
    memory: 512Mi
  limits:
    cpu: 1000m
    memory: 1Gi
```

### Redis Configuration

Modify redis.conf in the ConfigMap for custom Redis settings:

```yaml
data:
  redis.conf: |
    maxmemory 1gb
    maxmemory-policy allkeys-lru
    # Add more configurations
```

## Security Considerations

For production deployments, consider:

1. **Authentication**: Add Redis password authentication
2. **TLS**: Enable TLS for encrypted communication
3. **Network Policies**: Restrict network access to Redis pods
4. **RBAC**: Configure proper Kubernetes RBAC policies
5. **Pod Security**: Apply pod security policies/standards

Example password configuration:

```yaml
# In ConfigMap
data:
  redis.conf: |
    requirepass your-strong-password
    masterauth your-strong-password
```

## Cleanup

```bash
# Delete all resources
kubectl delete -k /workspace/infra/redis/

# Or delete namespace (removes everything)
kubectl delete namespace databases
```

## References

- [Redis Cluster Tutorial](https://redis.io/docs/management/scaling/)
- [Redis Sentinel Documentation](https://redis.io/docs/management/sentinel/)
- [Redis Configuration](https://redis.io/docs/management/config/)
