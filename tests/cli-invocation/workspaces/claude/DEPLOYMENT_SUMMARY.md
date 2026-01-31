# PostgreSQL Cluster Deployment Summary

## Created Files

### 1. postgresql-cluster.yaml
This manifest contains:
- **Namespace**: `databases` - Dedicated namespace for database resources
- **Cluster CR**: `alerthub-pg` - CloudNative-PG cluster with 3 instances
- **Secrets**: 
  - `alerthub-pg-superuser` - PostgreSQL superuser credentials
  - `alerthub-pg-backup-secret` - S3/MinIO credentials for backups
- **ConfigMap**: `alerthub-pg-monitoring` - Custom Prometheus metrics

### 2. postgresql-backup.yaml
This manifest contains:
- **Daily Backup**: Scheduled at 2:00 AM UTC
- **Weekly Backup**: Scheduled at 3:00 AM UTC (Sundays)
- **ConfigMap**: Backup policy documentation

### 3. deploy-and-verify.sh
Automated deployment and verification script

## Configuration Details

### Database Configuration
- **Database Name**: `alerthub`
- **Owner**: `alerthub_admin`
- **PostgreSQL Version**: 16.2
- **Instances**: 3 (1 primary + 2 replicas)

### Storage Configuration
- **Data Storage**: 20Gi with standard StorageClass
- **WAL Storage**: 5Gi with standard StorageClass
- **Access Mode**: ReadWriteOnce

### Resource Limits
- **Requests**: 500m CPU, 512Mi memory
- **Limits**: 2000m CPU, 2Gi memory

### Monitoring
- **Prometheus Scraping**: Enabled on port 9187
- **PodMonitor**: Enabled
- **Custom Metrics**: Database size tracking

### Backup Configuration
- **Backend**: S3-compatible (MinIO)
- **Endpoint**: http://minio.minio-system.svc.cluster.local:9000
- **Bucket**: alerthub-pg-backups
- **Compression**: gzip
- **Retention**: 30 days
- **WAL Archiving**: Continuous with gzip compression

### High Availability
- **Pod Anti-Affinity**: Spread replicas across different nodes
- **Update Strategy**: Unsupervised (automatic failover)
- **Replication**: Streaming replication enabled

## Deployment Instructions

### Prerequisites
1. CloudNative-PG operator must be installed:
   ```bash
   kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.22/releases/cnpg-1.22.0.yaml
   ```

2. MinIO or S3-compatible storage must be available for backups

### Deploy the Cluster

Option 1: Using the automated script
```bash
bash /workspace/deploy-and-verify.sh
```

Option 2: Manual deployment
```bash
# Apply cluster manifest
kubectl apply -f /workspace/postgresql-cluster.yaml

# Wait for cluster to be ready (may take several minutes)
kubectl wait --for=condition=ready cluster/alerthub-pg -n databases --timeout=600s

# Apply backup schedules
kubectl apply -f /workspace/postgresql-backup.yaml
```

## Verification Steps

### 1. Check Cluster Status
```bash
kubectl get cluster -n databases alerthub-pg
kubectl get pods -n databases -l postgresql=alerthub-pg
```

### 2. Verify PVCs are Bound
```bash
kubectl get pvc -n databases
```

### 3. Verify Database Exists
```bash
PRIMARY_POD=$(kubectl get pods -n databases -l postgresql=alerthub-pg,role=primary -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -c "\l" | grep alerthub
```

### 4. Test Database Connectivity
```bash
kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -d alerthub -c "SELECT version();"
```

### 5. Check Backup Schedules
```bash
kubectl get scheduledbackup -n databases
kubectl get backup -n databases
```

## Connection Information

### Service Endpoints
- **Primary (Read-Write)**: `alerthub-pg-rw.databases.svc.cluster.local:5432`
- **Replicas (Read-Only)**: `alerthub-pg-ro.databases.svc.cluster.local:5432`
- **Any Instance**: `alerthub-pg-r.databases.svc.cluster.local:5432`

### Get Credentials
```bash
# Get superuser password
kubectl get secret -n databases alerthub-pg-superuser -o jsonpath='{.data.password}' | base64 -d

# Get full credentials
kubectl get secret -n databases alerthub-pg-superuser -o yaml
```

### Access Database
```bash
# Interactive psql session
kubectl exec -it -n databases $PRIMARY_POD -- psql -U postgres -d alerthub

# Run single query
kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -d alerthub -c "SELECT current_database();"
```

## Acceptance Criteria Status

- ✅ PostgreSQL Cluster CR manifest created (`postgresql-cluster.yaml`)
- ✅ Backup schedule configuration created (`postgresql-backup.yaml`)
- ✅ Database name configured as `alerthub`
- ✅ Persistent storage configured with PVC (20Gi data + 5Gi WAL)
- ✅ Resource limits set (CPU/memory)
- ✅ Monitoring annotations configured for Prometheus scraping
- ✅ Backup policy configured with 30-day retention
- ⏳ Application to cluster pending (no cluster available in this environment)
- ⏳ Pod verification pending (requires cluster)
- ⏳ Database accessibility verification pending (requires cluster)
- ⏳ PVC binding verification pending (requires cluster)

## Next Steps

When a Kubernetes cluster is available:
1. Ensure CloudNative-PG operator is installed
2. Ensure MinIO or S3-compatible storage is configured
3. Run `bash /workspace/deploy-and-verify.sh` to deploy and verify
4. Monitor the cluster status and logs
5. Test backup and restore procedures

## Security Notes

- **Change default passwords** in production
- **Review and update** S3 credentials
- Consider using **External Secrets Operator** for secret management
- Enable **TLS/SSL** for database connections in production
- Implement **network policies** to restrict database access
- Regular **security updates** for PostgreSQL image

## Monitoring

The cluster exposes Prometheus metrics on port 9187:
- Standard PostgreSQL metrics
- Custom database size metrics
- Replication lag metrics
- Connection pool statistics

Query example:
```promql
# Database size
alerthub_custom_size_bytes

# Connection count
pg_stat_database_numbackends{datname="alerthub"}
```
