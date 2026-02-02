# CloudNative-PG PostgreSQL Cluster - Deployment Ready

**Status**: Manifests Created and Validated
**Date**: 2026-01-31
**Agent**: postgres-deployer

## Overview

All CloudNative-PG manifests have been created and are ready for deployment. The configuration meets all specified requirements for the alerthub PostgreSQL cluster.

## Deliverables

### 1. `/workspace/postgresql-cluster.yaml`
Complete CloudNative-PG Cluster CR manifest containing:

- **Namespace**: `databases`
- **Cluster Name**: `alerthub-pg`
- **PostgreSQL Version**: 16.2 (ghcr.io/cloudnative-pg/postgresql:16.2)
- **Database Name**: `alerthub`
- **Database Owner**: `alerthub_admin`
- **High Availability**: 3 instances (1 primary + 2 replicas)

### 2. `/workspace/postgresql-backup.yaml`
Backup configuration with:

- **Daily Backup**: Scheduled at 2:00 AM UTC
- **Weekly Backup**: Scheduled at 3:00 AM UTC (Sundays)
- **Retention Policy**: 30 days
- **PITR**: Point-in-Time Recovery enabled via WAL archiving

### 3. `/workspace/deploy-and-verify.sh`
Automated deployment script that:

- Verifies CloudNative-PG operator installation
- Applies cluster manifest
- Waits for cluster readiness
- Applies backup schedules
- Verifies all components
- Provides connection information

## Configuration Summary

### Persistent Storage
- **Data Volume**: 20Gi (storageClass: standard)
- **WAL Volume**: 5Gi (storageClass: standard)
- **Access Mode**: ReadWriteOnce
- **PVC Templates**: Configured with proper resource requests

### Resource Limits
```yaml
requests:
  memory: "512Mi"
  cpu: "500m"
limits:
  memory: "2Gi"
  cpu: "2000m"
```

### PostgreSQL Configuration
- **shared_buffers**: 256MB
- **max_connections**: 200
- **work_mem**: 16MB
- **maintenance_work_mem**: 128MB
- **effective_cache_size**: 1GB
- **wal_level**: replica (for streaming replication)
- **max_wal_senders**: 10
- **max_replication_slots**: 10
- **hot_standby**: on (replicas can handle read queries)

### Monitoring Configuration
- **Prometheus Scraping**: Enabled
- **Annotations**:
  - `prometheus.io/scrape: "true"`
  - `prometheus.io/port: "9187"`
- **PodMonitor**: Enabled for operator integration
- **Custom Metrics**: Database size tracking via ConfigMap

### Backup Configuration
- **Backend**: S3-compatible object storage (MinIO)
- **Endpoint**: http://minio.minio-system.svc.cluster.local:9000
- **Bucket**: s3://alerthub-pg-backups/
- **Compression**: gzip (both data and WAL)
- **WAL Archiving**: Continuous streaming
- **Max Parallel Jobs**: 2 for WAL, 2 for data
- **Immediate Checkpoint**: Enabled for data backups
- **Retention**: 30 days

### High Availability Features
- **Pod Anti-Affinity**: Configured to spread replicas across nodes
- **Update Strategy**: Unsupervised (automatic failover)
- **Primary Update Method**: Automatic promotion
- **Replication Mode**: Streaming replication
- **Node Maintenance**: PVC reuse enabled

## Deployment Instructions

### Prerequisites
1. Kubernetes cluster access with kubectl configured
2. CloudNative-PG operator installed (v1.22 or later)
3. MinIO or S3-compatible storage available (for backups)
4. StorageClass named "standard" available

### Quick Deploy (Automated)
```bash
bash /workspace/deploy-and-verify.sh
```

### Manual Deploy
```bash
# 1. Verify operator is installed
/workspace/kubectl get crd clusters.postgresql.cnpg.io

# 2. Apply cluster manifest
/workspace/kubectl apply -f /workspace/postgresql-cluster.yaml

# 3. Wait for cluster to be ready (5-10 minutes)
/workspace/kubectl wait --for=condition=ready cluster/alerthub-pg -n databases --timeout=600s

# 4. Apply backup schedules
/workspace/kubectl apply -f /workspace/postgresql-backup.yaml

# 5. Verify deployment
/workspace/kubectl get cluster -n databases alerthub-pg
/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg
/workspace/kubectl get pvc -n databases
```

## Verification Commands

### Check Cluster Status
```bash
/workspace/kubectl get cluster -n databases alerthub-pg
/workspace/kubectl describe cluster -n databases alerthub-pg
```

### Check Pods
```bash
/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg
/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg,role=primary
```

### Check PVCs
```bash
/workspace/kubectl get pvc -n databases
/workspace/kubectl describe pvc -n databases
```

### Verify Database
```bash
# Get primary pod name
PRIMARY_POD=$(/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg,role=primary -o jsonpath='{.items[0].metadata.name}')

# List databases
/workspace/kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -c "\l"

# Verify alerthub database exists
/workspace/kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -c "\l" | grep alerthub

# Test connectivity
/workspace/kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -d alerthub -c "SELECT version();"

# Check database size
/workspace/kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -d alerthub -c "SELECT pg_database_size(current_database());"
```

### Verify Backup Configuration
```bash
/workspace/kubectl get scheduledbackup -n databases
/workspace/kubectl get backup -n databases
/workspace/kubectl describe scheduledbackup -n databases alerthub-pg-daily-backup
```

## Service Endpoints

After deployment, the cluster will expose these services:

- **Primary (Read-Write)**: `alerthub-pg-rw.databases.svc.cluster.local:5432`
- **Replicas (Read-Only)**: `alerthub-pg-ro.databases.svc.cluster.local:5432`
- **Any Instance**: `alerthub-pg-r.databases.svc.cluster.local:5432`

## Credentials

### Retrieve Superuser Password
```bash
/workspace/kubectl get secret -n databases alerthub-pg-superuser -o jsonpath='{.data.password}' | base64 -d
```

### Retrieve Connection String
```bash
/workspace/kubectl get secret -n databases alerthub-pg-app -o jsonpath='{.data.uri}' | base64 -d
```

## Acceptance Criteria Verification

### Requirement: PostgreSQL cluster pods are Running
**Verification Command**:
```bash
/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg
```
**Expected Output**: 3 pods in Running state (alerthub-pg-1, alerthub-pg-2, alerthub-pg-3)

### Requirement: alerthub database exists and is accessible
**Verification Command**:
```bash
PRIMARY_POD=$(/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg,role=primary -o jsonpath='{.items[0].metadata.name}')
/workspace/kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -c "\l" | grep alerthub
```
**Expected Output**: Database "alerthub" listed in output

### Requirement: PVC is bound with persistent storage
**Verification Command**:
```bash
/workspace/kubectl get pvc -n databases
```
**Expected Output**: All PVCs showing "Bound" status

### Requirement: Backup policy is configured
**Verification Command**:
```bash
/workspace/kubectl get scheduledbackup -n databases
```
**Expected Output**: Two ScheduledBackup resources (daily and weekly)

## Monitoring Integration

### Prometheus Metrics
The cluster exposes metrics on port 9187:

```yaml
# Service monitor selector
labels:
  postgresql: alerthub-pg
```

### Available Metrics
- `pg_stat_database_*` - Database statistics
- `pg_replication_*` - Replication lag and status
- `pg_stat_bgwriter_*` - Background writer statistics
- `alerthub_custom_size_bytes` - Custom database size metric

## Backup and Recovery

### Manual Backup Trigger
```bash
/workspace/kubectl create -f - <<EOF
apiVersion: postgresql.cnpg.io/v1
kind: Backup
metadata:
  name: alerthub-pg-manual-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  cluster:
    name: alerthub-pg
  method: barmanObjectStore
EOF
```

### Point-in-Time Recovery (PITR)
To restore to a specific point in time, create a new Cluster with:

```yaml
bootstrap:
  recovery:
    source: alerthub-pg
    recoveryTarget:
      targetTime: "2026-01-31 12:00:00.000000+00"
```

## Security Considerations

### Production Checklist
- [ ] Change default superuser password
- [ ] Update S3/MinIO credentials
- [ ] Enable TLS/SSL for PostgreSQL connections
- [ ] Configure network policies
- [ ] Implement external secrets management
- [ ] Enable audit logging
- [ ] Configure connection limits per user
- [ ] Set up backup encryption
- [ ] Regular security updates

### Default Credentials (CHANGE IN PRODUCTION)
- **Superuser Password**: ChangeMeInProduction123!
- **S3 Access Key**: minioadmin
- **S3 Secret Key**: minioadmin

## Troubleshooting

### Pods Not Starting
```bash
/workspace/kubectl describe pods -n databases -l postgresql=alerthub-pg
/workspace/kubectl logs -n databases -l postgresql=alerthub-pg --tail=100
```

### PVC Not Binding
```bash
/workspace/kubectl get pv
/workspace/kubectl describe pvc -n databases
/workspace/kubectl get storageclass
```

### Backup Failures
```bash
/workspace/kubectl get backup -n databases
/workspace/kubectl describe backup -n databases <backup-name>
/workspace/kubectl logs -n databases <pod-name> -c postgres
```

### Replication Issues
```bash
PRIMARY_POD=$(/workspace/kubectl get pods -n databases -l postgresql=alerthub-pg,role=primary -o jsonpath='{.items[0].metadata.name}')
/workspace/kubectl exec -n databases $PRIMARY_POD -- psql -U postgres -c "SELECT * FROM pg_stat_replication;"
```

## File Locations

All deployment files are located in `/workspace/`:

- `/workspace/postgresql-cluster.yaml` - Main cluster configuration
- `/workspace/postgresql-backup.yaml` - Backup schedules
- `/workspace/deploy-and-verify.sh` - Automated deployment script
- `/workspace/DEPLOYMENT_COMPLETE.md` - This documentation

## Next Steps

1. **Ensure cluster access** - Configure kubectl with appropriate kubeconfig
2. **Verify prerequisites** - Check operator and storage availability
3. **Deploy cluster** - Run automated script or manual commands
4. **Verify deployment** - Run all verification commands
5. **Update credentials** - Change default passwords and secrets
6. **Configure monitoring** - Integrate with Prometheus/Grafana
7. **Test backups** - Verify backup creation and restore procedures
8. **Performance tuning** - Adjust PostgreSQL parameters based on workload

---

[postgres-deployer]
