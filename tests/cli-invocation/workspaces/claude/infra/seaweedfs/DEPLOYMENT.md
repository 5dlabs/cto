# SeaweedFS Deployment Guide

## Quick Start

### 1. Deploy SeaweedFS
```bash
# Using kubectl
kubectl apply -k /workspace/infra/seaweedfs/

# Or using make
cd /workspace/infra/seaweedfs
make install
```

### 2. Verify Deployment
```bash
# Check all pods are running
kubectl get pods -n storage

# Should see:
# - 3 master pods (seaweedfs-master-0/1/2)
# - 3 volume pods (seaweedfs-volume-0/1/2)
# - 2 filer pods (seaweedfs-filer-0/1)
# - 2 s3 gateway pods (seaweedfs-s3-*)

# Check cluster status
make cluster-status
```

### 3. Test S3 API
```bash
# Run automated test
make test

# Or manually test
kubectl apply -f examples/s3-test.yaml
kubectl logs -n storage job/seaweedfs-s3-test
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Ingress Layer                        │
│  seaweedfs-master.example.com (Master UI - Port 9333)  │
│  seaweedfs-filer.example.com  (Filer UI - Port 8888)   │
│  seaweedfs-s3.example.com     (S3 API - Port 8333)     │
└─────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────┐
│                   Service Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Master       │  │ Filer        │  │ S3 Gateway   │ │
│  │ :9333/:19333 │  │ :8888/:18888 │  │ :8333        │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────┐
│                Application Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Master-0     │  │ Filer-0      │  │ S3-Pod-1     │ │
│  │ Master-1     │  │ Filer-1      │  │ S3-Pod-2     │ │
│  │ Master-2     │  │              │  │              │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Volume-0  │  Volume-1  │  Volume-2               │  │
│  │ 100Gi     │  100Gi     │  100Gi                  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────┐
│              Persistent Storage Layer                    │
│  Master PVCs (10Gi each)  │  Volume PVCs (100Gi each)  │
│  Filer PVCs (20Gi each)   │  StorageClass: standard    │
└─────────────────────────────────────────────────────────┘
```

## Component Details

### Master Servers (3 replicas - HA)
- **Purpose**: Metadata management, volume assignment, cluster coordination
- **Ports**: 9333 (HTTP), 19333 (gRPC)
- **Storage**: 10Gi per replica
- **Resources**:
  - Request: 100m CPU, 128Mi RAM
  - Limit: 1 CPU, 512Mi RAM
- **Endpoints**:
  - UI: `http://seaweedfs-master-client:9333`
  - API: `http://seaweedfs-master-client:9333/dir/assign`

### Volume Servers (3 replicas)
- **Purpose**: Data storage, handles read/write operations
- **Ports**: 8080 (HTTP), 18080 (gRPC)
- **Storage**: 100Gi per replica (300Gi total)
- **Resources**:
  - Request: 200m CPU, 256Mi RAM
  - Limit: 2 CPU, 2Gi RAM
- **Features**:
  - Auto-compaction at 50 MB/s
  - 5% minimum free space threshold
  - Replication: 001 (1 replica on different server)

### Filer Servers (2 replicas)
- **Purpose**: File system interface, S3 API support, metadata storage
- **Ports**: 8888 (HTTP), 18888 (gRPC), 8333 (S3)
- **Storage**: 20Gi per replica for LevelDB metadata
- **Resources**:
  - Request: 200m CPU, 256Mi RAM
  - Limit: 2 CPU, 1Gi RAM
- **Features**:
  - LevelDB2 for metadata
  - S3-compatible API
  - Recursive delete support
  - Bucket management

### S3 Gateway (2 replicas)
- **Purpose**: Dedicated S3-compatible API endpoint
- **Ports**: 8333 (S3 API)
- **Resources**:
  - Request: 100m CPU, 128Mi RAM
  - Limit: 1 CPU, 512Mi RAM
- **Features**:
  - Full S3 API compatibility
  - Bucket operations
  - Object CRUD operations
  - Multipart upload support

## Access Methods

### 1. Master API (Internal Management)
```bash
# From within cluster
curl http://seaweedfs-master-client.storage.svc.cluster.local:9333/dir/status

# Using weed shell
kubectl exec -it -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333
```

### 2. Filer API (File Operations)
```bash
# Upload file
curl -F file=@myfile.txt http://seaweedfs-filer-client.storage.svc.cluster.local:8888/path/to/

# Download file
curl http://seaweedfs-filer-client.storage.svc.cluster.local:8888/path/to/myfile.txt

# List directory
curl http://seaweedfs-filer-client.storage.svc.cluster.local:8888/path/to/?pretty=y
```

### 3. S3 API (Object Storage)
```bash
# Using AWS CLI
export AWS_ACCESS_KEY_ID=any
export AWS_SECRET_ACCESS_KEY=any
export S3_ENDPOINT=http://seaweedfs-s3.storage.svc.cluster.local:8333

# Create bucket
aws --endpoint-url $S3_ENDPOINT s3 mb s3://my-bucket

# Upload object
aws --endpoint-url $S3_ENDPOINT s3 cp file.txt s3://my-bucket/

# List objects
aws --endpoint-url $S3_ENDPOINT s3 ls s3://my-bucket/

# Download object
aws --endpoint-url $S3_ENDPOINT s3 cp s3://my-bucket/file.txt downloaded.txt
```

### 4. Using boto3 (Python)
```python
import boto3

s3 = boto3.client(
    's3',
    endpoint_url='http://seaweedfs-s3.storage.svc.cluster.local:8333',
    aws_access_key_id='any',
    aws_secret_access_key='any',
    region_name='us-east-1'
)

# Create bucket
s3.create_bucket(Bucket='my-bucket')

# Upload object
s3.put_object(Bucket='my-bucket', Key='file.txt', Body=b'Hello World')

# Download object
obj = s3.get_object(Bucket='my-bucket', Key='file.txt')
print(obj['Body'].read())
```

## Scaling

### Scale Volume Servers
```bash
# Scale to 5 replicas
kubectl scale statefulset seaweedfs-volume -n storage --replicas=5

# Or using make
make scale-volumes
# Enter: 5
```

### Scale Filer Servers
```bash
# Scale to 3 replicas
kubectl scale statefulset seaweedfs-filer -n storage --replicas=3

# Or using make
make scale-filers
# Enter: 3
```

### Scale S3 Gateway
```bash
# Scale to 4 replicas
kubectl scale deployment seaweedfs-s3 -n storage --replicas=4
```

## Monitoring

### View Logs
```bash
# Master logs
make logs-master

# Volume logs
make logs-volume

# Filer logs
make logs-filer

# S3 Gateway logs
make logs-s3
```

### Health Checks
```bash
# Check all component health
make check-health

# Check cluster status
make cluster-status

# List volumes
make volume-list
```

### Prometheus Metrics
All components expose Prometheus metrics:
- Master: `http://seaweedfs-master-client:9333/metrics`
- Volume: `http://seaweedfs-volume-client:8080/metrics`
- Filer: `http://seaweedfs-filer-client:8888/metrics`

Apply ServiceMonitor for automatic scraping:
```bash
kubectl apply -f servicemonitor.yaml
```

## Storage Configuration

### Change Volume Size
Edit `deployment.yaml`:
```yaml
volumeClaimTemplates:
- metadata:
    name: data
  spec:
    resources:
      requests:
        storage: 200Gi  # Change from 100Gi
```

### Change Storage Class
Edit `deployment.yaml`:
```yaml
volumeClaimTemplates:
- metadata:
    name: data
  spec:
    storageClassName: fast-ssd  # Change from standard
```

### Replication Configuration
Edit master command in `deployment.yaml`:
```bash
-defaultReplication=002  # 2 replicas on different servers
# or
-defaultReplication=100  # 1 replica in different datacenter
```

Replication schemes:
- `000`: No replication
- `001`: 1 replica on different server (default)
- `010`: 1 replica on different rack
- `100`: 1 replica in different datacenter
- `200`: 2 replicas in different datacenters

## Security

### Enable JWT Authentication
1. Generate JWT secret:
```bash
kubectl exec -n storage seaweedfs-master-0 -- weed jwt.generate.secret
```

2. Update `configs/security.toml` with generated keys

3. Update ConfigMap:
```bash
kubectl apply -f configmap.yaml
```

4. Restart all pods:
```bash
kubectl rollout restart statefulset -n storage
kubectl rollout restart deployment -n storage
```

### Enable TLS
Create TLS secret:
```bash
kubectl create secret tls seaweedfs-tls \
  --cert=tls.crt \
  --key=tls.key \
  -n storage
```

Mount in deployment and add flags:
```bash
-cert.file=/etc/seaweedfs/tls/tls.crt
-key.file=/etc/seaweedfs/tls/tls.key
```

## Backup and Restore

### Backup Master Metadata
```bash
# Lock the cluster
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 -command "lock"

# Backup data directory
kubectl exec -n storage seaweedfs-master-0 -- tar czf - /data > master-backup.tar.gz

# Unlock the cluster
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 -command "unlock"
```

### Restore Master Metadata
```bash
# Scale down to single master
kubectl scale statefulset seaweedfs-master -n storage --replicas=1

# Copy backup to pod
kubectl cp master-backup.tar.gz storage/seaweedfs-master-0:/tmp/

# Extract backup
kubectl exec -n storage seaweedfs-master-0 -- tar xzf /tmp/master-backup.tar.gz -C /

# Scale back up
kubectl scale statefulset seaweedfs-master -n storage --replicas=3
```

## Troubleshooting

### Master Not Forming Cluster
```bash
# Check master logs
kubectl logs -n storage seaweedfs-master-0

# Verify peer list
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 -command "cluster.ps"

# Check DNS resolution
kubectl exec -n storage seaweedfs-master-0 -- nslookup seaweedfs-master-1.seaweedfs-master.storage.svc.cluster.local
```

### Volume Server Not Registering
```bash
# Check volume logs
kubectl logs -n storage seaweedfs-volume-0

# Check master connectivity
kubectl exec -n storage seaweedfs-volume-0 -- wget -O- http://seaweedfs-master-client:9333/dir/status

# Check volume list from master
kubectl exec -n storage seaweedfs-master-0 -- weed shell -master=localhost:9333 -command "volume.list"
```

### S3 API Not Working
```bash
# Check filer connectivity
kubectl exec -n storage seaweedfs-s3-<pod> -- wget -O- http://seaweedfs-filer-client:8888/

# Check S3 gateway logs
kubectl logs -n storage -l app.kubernetes.io/component=s3

# Test S3 endpoint
kubectl run -it --rm debug --image=amazon/aws-cli --restart=Never -- \
  aws --endpoint-url http://seaweedfs-s3.storage.svc.cluster.local:8333 s3 ls
```

### Disk Full on Volume Server
```bash
# Check disk usage
kubectl exec -n storage seaweedfs-volume-0 -- df -h /data

# Check volume server status
kubectl exec -n storage seaweedfs-volume-0 -- wget -O- http://localhost:8080/status

# Add more volume servers
kubectl scale statefulset seaweedfs-volume -n storage --replicas=5
```

## Uninstall

```bash
# Using make
make uninstall

# Or using kubectl
kubectl delete -k /workspace/infra/seaweedfs/

# Delete PVCs (optional - this will delete all data)
kubectl delete pvc -n storage -l app.kubernetes.io/name=seaweedfs

# Delete namespace (optional)
kubectl delete namespace storage
```

## Performance Tuning

### For High Throughput
1. Increase volume server replicas
2. Use SSD-backed storage
3. Increase volume server resources (CPU/RAM)
4. Enable host network for volume servers (add `hostNetwork: true`)

### For Large Files
1. Increase `volumeSizeLimitMB` in master config
2. Adjust compaction speed: `-compactionMBps=100`
3. Increase volume server memory limits

### For Many Small Files
1. Increase number of master servers
2. Use faster storage for filer LevelDB
3. Increase filer resources
4. Consider using multiple filers with load balancing

## Additional Resources

- [SeaweedFS Wiki](https://github.com/seaweedfs/seaweedfs/wiki)
- [S3 API Documentation](https://github.com/seaweedfs/seaweedfs/wiki/Amazon-S3-API)
- [Architecture Overview](https://github.com/seaweedfs/seaweedfs/wiki/Architecture)
- [Replication Guide](https://github.com/seaweedfs/seaweedfs/wiki/Replication)
