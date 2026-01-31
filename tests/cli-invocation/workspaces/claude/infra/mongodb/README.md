# MongoDB with Percona Operator Deployment

This directory contains Kubernetes manifests for deploying MongoDB using the Percona Server for MongoDB Operator.

## Architecture

- **Cluster Name**: mongodb-rs
- **Namespace**: databases
- **MongoDB Version**: 7.0.4-2
- **Replica Set**: 3-member high availability configuration
- **Storage**: 20Gi per member
- **Backup**: Daily logical and weekly physical backups
- **Monitoring**: PMM (Percona Monitoring and Management) integration

## Prerequisites

1. **Install Percona MongoDB Operator**:
   ```bash
   # Install the operator
   kubectl apply -f https://raw.githubusercontent.com/percona/percona-server-mongodb-operator/v1.16.0/deploy/bundle.yaml

   # Verify operator is running
   kubectl get pods -n default | grep percona-server-mongodb-operator
   ```

2. **Create the databases namespace**:
   ```bash
   kubectl apply -f namespace.yaml
   ```

3. **Configure Secrets**:

   Before deploying, update the secrets in `secrets.yaml`:

   ```bash
   # Generate secure passwords
   openssl rand -base64 32

   # Edit secrets.yaml with secure passwords
   vim secrets.yaml

   # Apply secrets
   kubectl apply -f secrets.yaml
   ```

## Files Overview

- **namespace.yaml**: Creates the `databases` namespace
- **secrets.yaml**: MongoDB user credentials and backup storage credentials
- **configmap.yaml**: MongoDB configuration settings
- **cluster.yaml**: Main PerconaServerMongoDB custom resource
- **service.yaml**: Kubernetes services for cluster access
- **kustomization.yaml**: Kustomize configuration for deployment management

## Deployment

### Option 1: Deploy with kubectl

```bash
# Deploy all resources
kubectl apply -f namespace.yaml
kubectl apply -f secrets.yaml
kubectl apply -f configmap.yaml
kubectl apply -f cluster.yaml
kubectl apply -f service.yaml

# Verify deployment
kubectl get psmdb -n databases
kubectl get pods -n databases -l app=mongodb
```

### Option 2: Deploy with Kustomize

```bash
# Deploy using kustomize
kubectl apply -k .

# Or with kustomize build
kustomize build . | kubectl apply -f -
```

## Monitoring Deployment

```bash
# Watch pod creation
kubectl get pods -n databases -w

# Check cluster status
kubectl get psmdb mongodb-rs -n databases

# View logs
kubectl logs -n databases -l app.kubernetes.io/instance=mongodb-rs -f

# Get detailed status
kubectl describe psmdb mongodb-rs -n databases
```

## Connection Information

### Connection Strings

**Internal Cluster Access**:
```
mongodb://databaseAdmin:PASSWORD@mongodb-rs-rs0-0.mongodb-rs-rs0.databases.svc.cluster.local:27017,mongodb-rs-rs0-1.mongodb-rs-rs0.databases.svc.cluster.local:27017,mongodb-rs-rs0-2.mongodb-rs-rs0.databases.svc.cluster.local:27017/admin?replicaSet=rs0
```

**Service-based Access** (recommended):
```
mongodb://databaseAdmin:PASSWORD@mongodb-rs.databases.svc.cluster.local:27017/admin?replicaSet=rs0
```

### Get Password

```bash
kubectl get secret mongodb-rs-secrets -n databases -o jsonpath='{.data.MONGODB_DATABASE_ADMIN_PASSWORD}' | base64 -d
```

### Connect from Pod

```bash
# Run MongoDB shell
kubectl run -i --rm --tty percona-client --image=percona/percona-server-mongodb:7.0.4-2 --restart=Never -n databases -- bash

# Inside the pod
mongosh "mongodb://mongodb-rs.databases.svc.cluster.local:27017/admin" -u databaseAdmin -p
```

## Backup Configuration

### S3 Backup Setup

1. **Configure S3 credentials** in `secrets.yaml`:
   ```yaml
   AWS_ACCESS_KEY_ID: "your-access-key-id"
   AWS_SECRET_ACCESS_KEY: "your-secret-access-key"
   ```

2. **Update S3 bucket** in `cluster.yaml`:
   ```yaml
   s3:
     bucket: mongodb-backups
     region: us-east-1
     prefix: mongodb-rs
   ```

3. **Apply configuration**:
   ```bash
   kubectl apply -f secrets.yaml
   kubectl apply -f cluster.yaml
   ```

### Backup Schedules

- **Daily Backup**: 3:00 AM (logical backup, 7 days retention)
- **Weekly Backup**: 4:00 AM Sunday (physical backup, 4 weeks retention)
- **PITR**: Enabled with gzip compression

### Manual Backup

```bash
# Create a backup resource
cat <<EOF | kubectl apply -f -
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBBackup
metadata:
  name: manual-backup-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  clusterName: mongodb-rs
  storageName: s3-backup
EOF

# Check backup status
kubectl get psmdb-backup -n databases
```

### Restore from Backup

```bash
# Create a restore resource
cat <<EOF | kubectl apply -f -
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBRestore
metadata:
  name: restore-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  clusterName: mongodb-rs
  backupName: backup-name-here
EOF

# Monitor restore
kubectl get psmdb-restore -n databases -w
```

## PMM (Percona Monitoring and Management)

### Setup

1. **Deploy PMM Server** (if not already deployed):
   ```bash
   kubectl apply -f https://raw.githubusercontent.com/percona/pmm/main/deploy/pmm-server.yaml
   ```

2. **Update PMM configuration** in `cluster.yaml`:
   ```yaml
   pmm:
     enabled: true
     serverHost: pmm-server.monitoring.svc.cluster.local
   ```

3. **Access PMM Dashboard**:
   ```bash
   kubectl port-forward -n monitoring svc/pmm-server 8443:443
   # Open https://localhost:8443
   ```

## Scaling

### Scale Replica Set

```bash
# Edit cluster.yaml and change size
kubectl patch psmdb mongodb-rs -n databases --type='json' -p='[{"op": "replace", "path": "/spec/replsets/0/size", "value":5}]'

# Or edit directly
kubectl edit psmdb mongodb-rs -n databases
```

### Increase Storage

```bash
# Edit PVC (requires storage class support for volume expansion)
kubectl patch pvc mongod-data-mongodb-rs-rs0-0 -n databases -p '{"spec":{"resources":{"requests":{"storage":"50Gi"}}}}'
```

## Maintenance

### Update MongoDB Version

```bash
# Edit cluster.yaml image version
kubectl patch psmdb mongodb-rs -n databases --type='json' -p='[{"op": "replace", "path": "/spec/image", "value":"percona/percona-server-mongodb:7.0.5-3"}]'
```

### Enable/Disable Backups

```bash
# Disable backups
kubectl patch psmdb mongodb-rs -n databases --type='json' -p='[{"op": "replace", "path": "/spec/backup/enabled", "value":false}]'

# Enable backups
kubectl patch psmdb mongodb-rs -n databases --type='json' -p='[{"op": "replace", "path": "/spec/backup/enabled", "value":true}]'
```

### Restart Pods

```bash
# Rolling restart
kubectl rollout restart statefulset mongodb-rs-rs0 -n databases

# Or delete pods one by one (operator will recreate)
kubectl delete pod mongodb-rs-rs0-0 -n databases
```

## Troubleshooting

### Check Operator Logs

```bash
kubectl logs -n default -l app.kubernetes.io/name=percona-server-mongodb-operator -f
```

### Check MongoDB Logs

```bash
kubectl logs -n databases mongodb-rs-rs0-0 -c mongod
```

### Check Cluster Status

```bash
# Get cluster status
kubectl get psmdb mongodb-rs -n databases -o yaml

# Check events
kubectl get events -n databases --sort-by='.lastTimestamp'
```

### Common Issues

1. **Pods not starting**: Check PVC binding
   ```bash
   kubectl get pvc -n databases
   kubectl describe pvc mongod-data-mongodb-rs-rs0-0 -n databases
   ```

2. **Backup failures**: Check S3 credentials and permissions
   ```bash
   kubectl logs -n databases mongodb-rs-rs0-0 -c backup-agent
   ```

3. **PMM not connecting**: Verify PMM server endpoint
   ```bash
   kubectl logs -n databases mongodb-rs-rs0-0 -c pmm-client
   ```

## Security Best Practices

1. **Change default passwords** before production deployment
2. **Use sealed secrets** or external secret management (Vault, AWS Secrets Manager)
3. **Enable network policies** to restrict access
4. **Configure TLS/SSL** for encrypted connections
5. **Regular backup testing** and disaster recovery drills
6. **Monitor and audit** database access logs
7. **Keep operator and MongoDB versions updated**

## Production Checklist

- [ ] Secure passwords configured
- [ ] S3 backup storage configured and tested
- [ ] PMM monitoring enabled and accessible
- [ ] Resource requests/limits tuned for workload
- [ ] Network policies configured
- [ ] TLS/SSL certificates configured
- [ ] Backup restore tested
- [ ] Disaster recovery plan documented
- [ ] Monitoring alerts configured
- [ ] High availability tested (pod failures)

## Resources

- [Percona Operator Documentation](https://docs.percona.com/percona-operator-for-mongodb/)
- [MongoDB Documentation](https://docs.mongodb.com/)
- [Percona Monitoring and Management](https://docs.percona.com/percona-monitoring-and-management/)
- [Backup and Restore Guide](https://docs.percona.com/percona-operator-for-mongodb/backups.html)

## Support

For issues and questions:
- [Percona Forums](https://forums.percona.com/)
- [GitHub Issues](https://github.com/percona/percona-server-mongodb-operator/issues)
- [Percona Support](https://www.percona.com/services/support)
