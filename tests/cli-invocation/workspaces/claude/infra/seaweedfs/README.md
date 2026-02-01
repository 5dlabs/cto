# SeaweedFS Object Storage

This directory contains Kubernetes manifests for deploying SeaweedFS, a distributed object storage system with S3-compatible API.

## Architecture

SeaweedFS consists of the following components:

### Master Servers (3 replicas)
- Manage metadata and volume assignment
- Coordinate cluster operations
- HA configuration with 3 nodes
- Resources: 100m CPU, 128Mi RAM (request) / 1 CPU, 512Mi RAM (limit)

### Volume Servers (3 replicas)
- Store actual data files
- 100Gi persistent storage each
- Handle read/write operations
- Resources: 200m CPU, 256Mi RAM (request) / 2 CPU, 2Gi RAM (limit)

### Filer Servers (2 replicas)
- Provide file system interface
- Support S3-compatible API
- LevelDB for metadata storage
- Resources: 200m CPU, 256Mi RAM (request) / 2 CPU, 1Gi RAM (limit)

### S3 Gateway (2 replicas)
- Dedicated S3-compatible API endpoint
- Connects to filer for data access
- Resources: 100m CPU, 128Mi RAM (request) / 1 CPU, 512Mi RAM (limit)

## Configuration

### ConfigMap
- `master.toml`: Master server configuration
- `filer.toml`: Filer and S3 settings
- `security.toml`: JWT and security settings

### Storage
- Volume servers: 100Gi per instance
- Filer servers: 20Gi per instance
- Master servers: 10Gi per instance
- StorageClass: `standard` (customize as needed)

## Services

### Internal Services
- `seaweedfs-master:9333` - Master API
- `seaweedfs-volume:8080` - Volume server API
- `seaweedfs-filer:8888` - Filer API
- `seaweedfs-s3:8333` - S3-compatible API

### Client Services
- `seaweedfs-master-client:9333` - Master client access
- `seaweedfs-volume-client:8080` - Volume client access
- `seaweedfs-filer-client:8888` - Filer client access

## Deployment

### Apply manifests
```bash
kubectl apply -k /workspace/infra/seaweedfs/
```

### Verify deployment
```bash
# Check all pods are running
kubectl get pods -n storage

# Check master cluster status
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 cluster.status

# Check volume servers
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 volume.list
```

## S3 Access

### Endpoint
- Internal: `http://seaweedfs-s3.storage.svc.cluster.local:8333`
- External: Configure ingress at `seaweedfs-s3.example.com`

### AWS CLI Configuration
```bash
aws configure set aws_access_key_id any
aws configure set aws_secret_access_key any
aws configure set default.region us-east-1

# Create bucket
aws --endpoint-url http://seaweedfs-s3.example.com s3 mb s3://my-bucket

# Upload file
aws --endpoint-url http://seaweedfs-s3.example.com s3 cp file.txt s3://my-bucket/

# List objects
aws --endpoint-url http://seaweedfs-s3.example.com s3 ls s3://my-bucket/
```

## Replication

Current configuration uses `001` replication:
- `0`: No replication on different data centers
- `0`: No replication on different racks
- `1`: 1 replica on different servers

To modify replication, update:
- Master: `-defaultReplication` flag
- Filer: `-defaultReplicaPlacement` flag

Common replication schemes:
- `000`: No replication
- `001`: 1 replica on different server (default)
- `010`: 1 replica on different rack
- `100`: 1 replica in different data center
- `200`: 2 replicas in different data centers

## Scaling

### Scale volume servers
```bash
kubectl scale statefulset seaweedfs-volume -n storage --replicas=5
```

### Scale filer servers
```bash
kubectl scale statefulset seaweedfs-filer -n storage --replicas=3
```

### Scale S3 gateway
```bash
kubectl scale deployment seaweedfs-s3 -n storage --replicas=3
```

## Monitoring

### Master UI
Access at: `http://seaweedfs-master-client.storage.svc.cluster.local:9333`

### Filer UI
Access at: `http://seaweedfs-filer-client.storage.svc.cluster.local:8888`

### Metrics
SeaweedFS exposes Prometheus metrics at:
- Master: `:9333/metrics`
- Volume: `:8080/metrics`
- Filer: `:8888/metrics`

## Backup and Restore

### Backup metadata
```bash
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 lock
kubectl exec -n storage seaweedfs-master-0 -- tar czf - /data > master-backup.tar.gz
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 unlock
```

### Backup volumes
```bash
# Use volume.tier.move command to move to cold storage
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333
> volume.tier.download -volumeId=1
```

## Security

### Enable JWT Authentication
1. Generate secret keys in `security.toml`
2. Update ConfigMap
3. Restart all pods

### Enable TLS
Add TLS certificates as secrets and mount to pods:
```yaml
- name: tls
  secret:
    secretName: seaweedfs-tls
```

Update command flags:
```bash
-cert.file=/etc/seaweedfs/tls/tls.crt
-key.file=/etc/seaweedfs/tls/tls.key
```

## Troubleshooting

### Check master leader
```bash
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 master.maintenance.get.leader
```

### Check volume health
```bash
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 volume.check.disk
```

### View logs
```bash
kubectl logs -n storage -l app.kubernetes.io/component=master -f
kubectl logs -n storage -l app.kubernetes.io/component=volume -f
kubectl logs -n storage -l app.kubernetes.io/component=filer -f
```

## Best Practices

1. **Resource Allocation**: Adjust CPU/memory based on workload
2. **Storage Class**: Use SSD-backed storage for better performance
3. **Anti-affinity**: Enabled to spread pods across nodes
4. **Monitoring**: Set up Prometheus scraping for metrics
5. **Backup**: Regular backups of master metadata
6. **Replication**: Choose appropriate replication level for durability
7. **Compaction**: Volume servers auto-compact with `-compactionMBps=50`
8. **Network**: Use host network for better volume server performance (optional)

## References

- [SeaweedFS Documentation](https://github.com/seaweedfs/seaweedfs/wiki)
- [S3 API Compatibility](https://github.com/seaweedfs/seaweedfs/wiki/Amazon-S3-API)
- [Architecture](https://github.com/seaweedfs/seaweedfs/wiki/Architecture)
