#!/bin/bash

# PostgreSQL CloudNative-PG Cluster Verification Script
# This script verifies the health and configuration of the PostgreSQL cluster

set -e

NAMESPACE="databases"
CLUSTER_NAME="postgres-cluster"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "PostgreSQL Cluster Verification Script"
echo "=========================================="
echo ""

# Function to check command status
check_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $1"
    else
        echo -e "${RED}✗${NC} $1"
        return 1
    fi
}

# 1. Check if namespace exists
echo "1. Checking namespace..."
kubectl get namespace $NAMESPACE &> /dev/null
check_status "Namespace '$NAMESPACE' exists"
echo ""

# 2. Check if CloudNative-PG operator is installed
echo "2. Checking CloudNative-PG operator..."
kubectl get deployment -n cnpg-system cnpg-controller-manager &> /dev/null
check_status "CloudNative-PG operator is installed"
echo ""

# 3. Check cluster status
echo "3. Checking cluster status..."
kubectl get cluster $CLUSTER_NAME -n $NAMESPACE &> /dev/null
check_status "Cluster '$CLUSTER_NAME' exists"

CLUSTER_STATUS=$(kubectl get cluster $CLUSTER_NAME -n $NAMESPACE -o jsonpath='{.status.phase}' 2>/dev/null)
if [ "$CLUSTER_STATUS" == "Cluster in healthy state" ]; then
    echo -e "${GREEN}✓${NC} Cluster is healthy"
else
    echo -e "${YELLOW}!${NC} Cluster status: $CLUSTER_STATUS"
fi

INSTANCES=$(kubectl get cluster $CLUSTER_NAME -n $NAMESPACE -o jsonpath='{.status.instances}' 2>/dev/null)
READY_INSTANCES=$(kubectl get cluster $CLUSTER_NAME -n $NAMESPACE -o jsonpath='{.status.readyInstances}' 2>/dev/null)
echo "   Instances: $READY_INSTANCES/$INSTANCES ready"
echo ""

# 4. Check pods
echo "4. Checking PostgreSQL pods..."
PODS=$(kubectl get pods -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME --no-headers 2>/dev/null | wc -l)
RUNNING_PODS=$(kubectl get pods -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME --field-selector status.phase=Running --no-headers 2>/dev/null | wc -l)
echo "   Running pods: $RUNNING_PODS/$PODS"

kubectl get pods -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME --no-headers 2>/dev/null | while read pod rest; do
    STATUS=$(kubectl get pod $pod -n $NAMESPACE -o jsonpath='{.status.phase}' 2>/dev/null)
    ROLE=$(kubectl get pod $pod -n $NAMESPACE -o jsonpath='{.metadata.labels.cnpg\.io/instanceRole}' 2>/dev/null)
    if [ "$STATUS" == "Running" ]; then
        echo -e "   ${GREEN}✓${NC} $pod ($ROLE)"
    else
        echo -e "   ${RED}✗${NC} $pod ($ROLE) - Status: $STATUS"
    fi
done
echo ""

# 5. Check services
echo "5. Checking services..."
for service in "${CLUSTER_NAME}-rw" "${CLUSTER_NAME}-ro" "${CLUSTER_NAME}-r"; do
    kubectl get service $service -n $NAMESPACE &> /dev/null
    check_status "Service '$service' exists"
done
echo ""

# 6. Check poolers
echo "6. Checking connection poolers..."
kubectl get pooler ${CLUSTER_NAME}-pooler-rw -n $NAMESPACE &> /dev/null
check_status "RW Pooler exists"

kubectl get pooler ${CLUSTER_NAME}-pooler-ro -n $NAMESPACE &> /dev/null
check_status "RO Pooler exists"

POOLER_RW_PODS=$(kubectl get pods -n $NAMESPACE -l cnpg.io/poolerName=${CLUSTER_NAME}-pooler-rw --field-selector status.phase=Running --no-headers 2>/dev/null | wc -l)
POOLER_RO_PODS=$(kubectl get pods -n $NAMESPACE -l cnpg.io/poolerName=${CLUSTER_NAME}-pooler-ro --field-selector status.phase=Running --no-headers 2>/dev/null | wc -l)
echo "   RW Pooler pods running: $POOLER_RW_PODS"
echo "   RO Pooler pods running: $POOLER_RO_PODS"
echo ""

# 7. Check PVCs
echo "7. Checking Persistent Volume Claims..."
PVCS=$(kubectl get pvc -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME --no-headers 2>/dev/null | wc -l)
BOUND_PVCS=$(kubectl get pvc -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME --field-selector status.phase=Bound --no-headers 2>/dev/null | wc -l)
echo "   Bound PVCs: $BOUND_PVCS/$PVCS"

kubectl get pvc -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME --no-headers 2>/dev/null | while read pvc rest; do
    STATUS=$(kubectl get pvc $pvc -n $NAMESPACE -o jsonpath='{.status.phase}' 2>/dev/null)
    SIZE=$(kubectl get pvc $pvc -n $NAMESPACE -o jsonpath='{.spec.resources.requests.storage}' 2>/dev/null)
    if [ "$STATUS" == "Bound" ]; then
        echo -e "   ${GREEN}✓${NC} $pvc ($SIZE)"
    else
        echo -e "   ${RED}✗${NC} $pvc - Status: $STATUS"
    fi
done
echo ""

# 8. Check secrets
echo "8. Checking secrets..."
for secret in "${CLUSTER_NAME}-app-user" "${CLUSTER_NAME}-superuser" "backup-storage-credentials"; do
    kubectl get secret $secret -n $NAMESPACE &> /dev/null
    check_status "Secret '$secret' exists"
done
echo ""

# 9. Check scheduled backups
echo "9. Checking scheduled backups..."
SCHEDULED_BACKUPS=$(kubectl get scheduledbackup -n $NAMESPACE --no-headers 2>/dev/null | wc -l)
if [ $SCHEDULED_BACKUPS -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Found $SCHEDULED_BACKUPS scheduled backup(s)"
    kubectl get scheduledbackup -n $NAMESPACE --no-headers 2>/dev/null | while read backup rest; do
        SCHEDULE=$(kubectl get scheduledbackup $backup -n $NAMESPACE -o jsonpath='{.spec.schedule}' 2>/dev/null)
        SUSPEND=$(kubectl get scheduledbackup $backup -n $NAMESPACE -o jsonpath='{.spec.suspend}' 2>/dev/null)
        echo "   - $backup: $SCHEDULE (suspended: $SUSPEND)"
    done
else
    echo -e "${YELLOW}!${NC} No scheduled backups found"
fi
echo ""

# 10. Check backups
echo "10. Checking backups..."
BACKUPS=$(kubectl get backup -n $NAMESPACE 2>/dev/null | grep -c $CLUSTER_NAME || true)
if [ $BACKUPS -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Found $BACKUPS backup(s)"
    kubectl get backup -n $NAMESPACE 2>/dev/null | grep $CLUSTER_NAME | head -5 | while read backup rest; do
        echo "   - $backup"
    done
else
    echo -e "${YELLOW}!${NC} No backups found yet (this is normal for new deployments)"
fi
echo ""

# 11. Check replication status
echo "11. Checking replication status..."
PRIMARY_POD=$(kubectl get pods -n $NAMESPACE -l cnpg.io/cluster=$CLUSTER_NAME,cnpg.io/instanceRole=primary --no-headers 2>/dev/null | awk '{print $1}')

if [ -n "$PRIMARY_POD" ]; then
    echo "   Primary pod: $PRIMARY_POD"

    REPLICATION_STATUS=$(kubectl exec -n $NAMESPACE $PRIMARY_POD -- psql -U postgres -t -c "SELECT count(*) FROM pg_stat_replication;" 2>/dev/null | xargs)
    if [ -n "$REPLICATION_STATUS" ] && [ "$REPLICATION_STATUS" -gt 0 ]; then
        echo -e "${GREEN}✓${NC} Replication active: $REPLICATION_STATUS replica(s) connected"

        # Show detailed replication info
        echo ""
        echo "   Replication Details:"
        kubectl exec -n $NAMESPACE $PRIMARY_POD -- psql -U postgres -c "SELECT application_name, state, sync_state, replay_lag FROM pg_stat_replication;" 2>/dev/null | sed 's/^/   /'
    else
        echo -e "${YELLOW}!${NC} No replicas connected (this may be normal during startup)"
    fi
else
    echo -e "${RED}✗${NC} Primary pod not found"
fi
echo ""

# 12. Test connectivity
echo "12. Testing database connectivity..."

# Test if we can connect to the database
TEST_RESULT=$(kubectl run test-connection-$$  -n $NAMESPACE --rm -i --restart=Never --image=postgres:16-alpine --command -- \
    sh -c "PGPASSWORD=\$(kubectl get secret ${CLUSTER_NAME}-app-user -n $NAMESPACE -o jsonpath='{.data.password}' | base64 -d) psql -h ${CLUSTER_NAME}-pooler-rw -U app -d app -c 'SELECT 1' -t" 2>/dev/null | xargs)

if [ "$TEST_RESULT" == "1" ]; then
    echo -e "${GREEN}✓${NC} Successfully connected to database via RW pooler"
else
    echo -e "${RED}✗${NC} Failed to connect to database"
fi
echo ""

# 13. Summary
echo "=========================================="
echo "Verification Summary"
echo "=========================================="
echo ""

# Calculate overall health
HEALTH_SCORE=0
TOTAL_CHECKS=0

# Simple health calculation
if [ "$CLUSTER_STATUS" == "Cluster in healthy state" ]; then ((HEALTH_SCORE++)); fi
((TOTAL_CHECKS++))

if [ $RUNNING_PODS -eq $PODS ] && [ $PODS -gt 0 ]; then ((HEALTH_SCORE++)); fi
((TOTAL_CHECKS++))

if [ $POOLER_RW_PODS -gt 0 ] && [ $POOLER_RO_PODS -gt 0 ]; then ((HEALTH_SCORE++)); fi
((TOTAL_CHECKS++))

if [ $BOUND_PVCS -eq $PVCS ] && [ $PVCS -gt 0 ]; then ((HEALTH_SCORE++)); fi
((TOTAL_CHECKS++))

if [ -n "$REPLICATION_STATUS" ] && [ "$REPLICATION_STATUS" -gt 0 ]; then ((HEALTH_SCORE++)); fi
((TOTAL_CHECKS++))

echo "Health Score: $HEALTH_SCORE/$TOTAL_CHECKS checks passed"
echo ""

if [ $HEALTH_SCORE -eq $TOTAL_CHECKS ]; then
    echo -e "${GREEN}✓ Cluster is fully operational!${NC}"
    exit 0
elif [ $HEALTH_SCORE -gt $((TOTAL_CHECKS / 2)) ]; then
    echo -e "${YELLOW}! Cluster is partially operational. Please review the checks above.${NC}"
    exit 1
else
    echo -e "${RED}✗ Cluster has significant issues. Please review the checks above.${NC}"
    exit 2
fi
