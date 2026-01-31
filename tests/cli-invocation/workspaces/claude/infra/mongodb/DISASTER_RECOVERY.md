# MongoDB Disaster Recovery Guide

This guide provides procedures for disaster recovery scenarios with Percona Server for MongoDB.

## Table of Contents

1. [Backup Strategy](#backup-strategy)
2. [Recovery Scenarios](#recovery-scenarios)
3. [Testing Procedures](#testing-procedures)
4. [Emergency Contacts](#emergency-contacts)

## Backup Strategy

### Automated Backups

The cluster is configured with:
- **Daily Logical Backups**: 3:00 AM, 7 days retention
- **Weekly Physical Backups**: 4:00 AM Sunday, 4 weeks retention
- **Point-in-Time Recovery (PITR)**: Enabled with continuous oplog backup

### Backup Locations

- **Primary**: S3 bucket `mongodb-backups/mongodb-rs/`
- **Backup Verification**: Automated weekly restore tests

## Recovery Scenarios

### Scenario 1: Single Pod Failure

**Detection**: Pod crashes or becomes unresponsive

**Impact**: Minimal - cluster continues operating with 2/3 nodes

**Recovery**:
```bash
# Check pod status
kubectl get pods -n databases -l app.kubernetes.io/instance=mongodb-rs

# View pod logs
kubectl logs -n databases mongodb-rs-rs0-X

# Delete failed pod (operator will recreate)
kubectl delete pod mongodb-rs-rs0-X -n databases

# Monitor recovery
kubectl get pods -n databases -w
```

**RTO**: 5-10 minutes
**RPO**: 0 (no data loss)

---

### Scenario 2: Storage Corruption

**Detection**: Database errors, corruption messages in logs

**Impact**: Single replica affected

**Recovery**:
```bash
# 1. Scale down the affected pod
kubectl delete pod mongodb-rs-rs0-X -n databases

# 2. Delete the corrupted PVC
kubectl delete pvc mongod-data-mongodb-rs-rs0-X -n databases

# 3. Operator will recreate pod and PVC
# 4. MongoDB will sync data from other replicas

# 5. Monitor sync progress
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "rs.status()"
```

**RTO**: 30-60 minutes (depends on data size)
**RPO**: 0 (no data loss with healthy replicas)

---

### Scenario 3: Complete Cluster Failure

**Detection**: All pods down, namespace deleted, or cluster destroyed

**Impact**: Complete service outage

**Recovery**:

#### Step 1: Restore Infrastructure
```bash
# Recreate namespace
kubectl apply -f namespace.yaml

# Recreate secrets
kubectl apply -f secrets.yaml

# Recreate configmap
kubectl apply -f configmap.yaml
```

#### Step 2: Restore from Latest Backup
```bash
# Find latest backup
kubectl get psmdb-backup -n databases

# Create restore resource
cat <<EOF | kubectl apply -f -
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBRestore
metadata:
  name: restore-emergency-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  clusterName: mongodb-rs
  backupName: <latest-backup-name>
  # Optionally restore to specific point in time
  # pitr:
  #   type: date
  #   date: "2024-01-15T10:30:00Z"
EOF

# Monitor restore progress
kubectl get psmdb-restore -n databases -w
kubectl logs -n databases -l app.kubernetes.io/component=backup -f
```

#### Step 3: Verify Data Integrity
```bash
# Connect to MongoDB
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "
  db.adminCommand({ listDatabases: 1 })
"

# Check collection counts
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh myapp --eval "
  db.getCollectionNames().forEach(function(col) {
    print(col + ': ' + db[col].count());
  });
"

# Verify replica set status
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "rs.status()"
```

**RTO**: 2-4 hours (depends on backup size)
**RPO**: Up to 24 hours (last daily backup) or minutes (with PITR)

---

### Scenario 4: Namespace Deletion

**Detection**: Namespace databases no longer exists

**Impact**: Complete service outage

**Recovery**:
```bash
# Redeploy everything
kubectl apply -k .

# Restore from backup (see Scenario 3)
```

**RTO**: 2-4 hours
**RPO**: Up to 24 hours or minutes (with PITR)

---

### Scenario 5: Accidental Data Deletion

**Detection**: Application reports missing data

**Impact**: Data loss in specific collections/documents

**Recovery**:

#### Option A: Point-in-Time Recovery (PITR)
```bash
# Restore to time before deletion
cat <<EOF | kubectl apply -f -
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBRestore
metadata:
  name: restore-pitr-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  clusterName: mongodb-rs
  backupName: <base-backup-name>
  pitr:
    type: date
    date: "2024-01-15T10:30:00Z"  # Time before deletion
EOF
```

#### Option B: Restore to Temporary Cluster
```bash
# 1. Create temporary cluster
kubectl apply -f - <<EOF
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDB
metadata:
  name: mongodb-temp
  namespace: databases
spec:
  # ... same config as main cluster
  replsets:
    - name: rs0
      size: 1  # Single node for temp recovery
EOF

# 2. Restore to temporary cluster
cat <<EOF | kubectl apply -f -
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBRestore
metadata:
  name: restore-temp-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  clusterName: mongodb-temp
  backupName: <backup-name>
EOF

# 3. Export missing data
kubectl exec -n databases mongodb-temp-rs0-0 -- mongodump \
  --db=myapp \
  --collection=mycollection \
  --out=/tmp/recovery

# 4. Import to main cluster
kubectl exec -n databases mongodb-rs-rs0-0 -- mongorestore \
  --db=myapp \
  /tmp/recovery/myapp

# 5. Cleanup temp cluster
kubectl delete psmdb mongodb-temp -n databases
```

**RTO**: 1-2 hours
**RPO**: Up to last backup interval

---

### Scenario 6: Network Partition

**Detection**: Replica set members cannot communicate

**Impact**: Possible split-brain, read-only operations

**Recovery**:
```bash
# 1. Check network connectivity
kubectl exec -n databases mongodb-rs-rs0-0 -- ping mongodb-rs-rs0-1.mongodb-rs-rs0.databases.svc.cluster.local

# 2. Check replica set status
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "rs.status()"

# 3. If split-brain detected, force reconfiguration
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "
  cfg = rs.conf();
  cfg.version++;
  rs.reconfig(cfg, {force: true});
"

# 4. Wait for cluster to stabilize
watch kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "rs.status()"
```

**RTO**: 15-30 minutes
**RPO**: 0 (no data loss if handled correctly)

---

### Scenario 7: Operator Failure

**Detection**: Operator pod not running, cluster not responding to changes

**Impact**: Unable to manage cluster, automated tasks fail

**Recovery**:
```bash
# 1. Check operator status
kubectl get pods -n default | grep percona-server-mongodb-operator

# 2. View operator logs
kubectl logs -n default -l app.kubernetes.io/name=percona-server-mongodb-operator

# 3. Restart operator
kubectl rollout restart deployment percona-server-mongodb-operator -n default

# 4. If operator is missing, reinstall
kubectl apply -f https://raw.githubusercontent.com/percona/percona-server-mongodb-operator/v1.16.0/deploy/bundle.yaml

# 5. Verify cluster status
kubectl get psmdb -n databases
```

**RTO**: 5-10 minutes
**RPO**: 0 (no data loss)

---

## Testing Procedures

### Monthly Backup Restore Test

```bash
#!/bin/bash
# Test backup restore to temporary cluster

# 1. Create test namespace
kubectl create namespace mongodb-test

# 2. Copy secrets
kubectl get secret mongodb-rs-secrets -n databases -o yaml | \
  sed 's/namespace: databases/namespace: mongodb-test/' | \
  kubectl apply -f -

kubectl get secret mongodb-backup-s3 -n databases -o yaml | \
  sed 's/namespace: databases/namespace: mongodb-test/' | \
  kubectl apply -f -

# 3. Create test cluster
kubectl apply -n mongodb-test -f - <<EOF
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDB
metadata:
  name: mongodb-test
spec:
  replsets:
    - name: rs0
      size: 1
      volumeSpec:
        persistentVolumeClaim:
          resources:
            requests:
              storage: 10Gi
  # ... minimal config
EOF

# 4. Restore latest backup
LATEST_BACKUP=$(kubectl get psmdb-backup -n databases -o jsonpath='{.items[0].metadata.name}')

kubectl apply -f - <<EOF
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBRestore
metadata:
  name: test-restore
  namespace: mongodb-test
spec:
  clusterName: mongodb-test
  backupName: ${LATEST_BACKUP}
EOF

# 5. Wait for restore
kubectl wait --for=condition=ready psmdb/mongodb-test -n mongodb-test --timeout=30m

# 6. Verify data
kubectl exec -n mongodb-test mongodb-test-rs0-0 -- mongosh --eval "
  db.adminCommand({ listDatabases: 1 })
"

# 7. Cleanup
kubectl delete namespace mongodb-test

echo "Backup restore test completed"
```

### Quarterly Disaster Recovery Drill

Full disaster recovery simulation:

1. **Schedule maintenance window**
2. **Document current state** (pod count, data size, connections)
3. **Delete cluster** (or simulate failure)
4. **Follow complete recovery procedure**
5. **Verify all data** and functionality
6. **Document RTO/RPO** achieved
7. **Update procedures** based on lessons learned

## Emergency Response Checklist

### Initial Response

- [ ] Assess scope and impact
- [ ] Check monitoring and alerts
- [ ] Notify stakeholders
- [ ] Start incident timeline
- [ ] Determine if emergency or planned maintenance

### During Recovery

- [ ] Follow appropriate scenario procedure
- [ ] Document all actions taken
- [ ] Monitor progress continuously
- [ ] Provide regular status updates
- [ ] Keep communication channels open

### Post-Recovery

- [ ] Verify data integrity
- [ ] Test application functionality
- [ ] Monitor for 24 hours
- [ ] Conduct post-mortem
- [ ] Update documentation
- [ ] Implement preventive measures

## Emergency Contacts

| Role | Name | Contact | Availability |
|------|------|---------|--------------|
| Database Admin | TBD | TBD | 24/7 |
| Platform Team | TBD | TBD | 24/7 |
| Application Team | TBD | TBD | Business hours |
| Percona Support | Support Portal | https://www.percona.com/services/support | 24/7 (with contract) |

## Useful Commands Reference

```bash
# Check cluster health
kubectl get psmdb mongodb-rs -n databases
kubectl exec -n databases mongodb-rs-rs0-0 -- mongosh --eval "rs.status()"

# View logs
kubectl logs -n databases mongodb-rs-rs0-0 -c mongod -f

# List backups
kubectl get psmdb-backup -n databases

# Check restore status
kubectl get psmdb-restore -n databases -w

# Force immediate backup
kubectl apply -f - <<EOF
apiVersion: psmdb.percona.com/v1
kind: PerconaServerMongoDBBackup
metadata:
  name: emergency-backup-$(date +%Y%m%d-%H%M%S)
  namespace: databases
spec:
  clusterName: mongodb-rs
  storageName: s3-backup
EOF

# Access MongoDB shell
kubectl exec -it -n databases mongodb-rs-rs0-0 -- mongosh

# Export specific collection
kubectl exec -n databases mongodb-rs-rs0-0 -- mongodump \
  --db=myapp --collection=mycollection --out=/tmp/backup

# Check storage usage
kubectl exec -n databases mongodb-rs-rs0-0 -- df -h /data/db
```

## Prevention Measures

1. **Regular Testing**: Monthly backup restore tests
2. **Monitoring**: Comprehensive alerts for all failure scenarios
3. **Automation**: Automated backup verification
4. **Documentation**: Keep runbooks up to date
5. **Training**: Quarterly DR drills
6. **Redundancy**: Multi-AZ deployment, multiple replicas
7. **Backup Strategy**: Multiple backup types and schedules
8. **Security**: Access controls and audit logging

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-01-15 | DevOps Team | Initial version |

---

**Last Updated**: 2024-01-15
**Next Review**: 2024-04-15
