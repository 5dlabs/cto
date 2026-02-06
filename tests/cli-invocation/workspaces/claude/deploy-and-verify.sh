#!/bin/bash

set -e

echo "=========================================="
echo "PostgreSQL Cluster Deployment Script"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check command status
check_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[SUCCESS]${NC} $1"
    else
        echo -e "${RED}[FAILED]${NC} $1"
        exit 1
    fi
}

# Function to wait for resource
wait_for_resource() {
    local resource=$1
    local namespace=$2
    local timeout=${3:-300}

    echo "Waiting for $resource in namespace $namespace (timeout: ${timeout}s)..."
    kubectl wait --for=condition=ready "$resource" -n "$namespace" --timeout="${timeout}s"
    check_status "Resource $resource is ready"
}

# Step 1: Verify CloudNative-PG operator is installed
echo -e "${YELLOW}Step 1: Verifying CloudNative-PG operator...${NC}"
if kubectl get crd clusters.postgresql.cnpg.io &> /dev/null; then
    echo -e "${GREEN}[OK]${NC} CloudNative-PG CRD found"
else
    echo -e "${RED}[ERROR]${NC} CloudNative-PG operator not installed"
    echo "Please install the operator first:"
    echo "  kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.22/releases/cnpg-1.22.0.yaml"
    exit 1
fi

# Step 2: Apply PostgreSQL cluster manifest
echo ""
echo -e "${YELLOW}Step 2: Applying PostgreSQL cluster manifest...${NC}"
kubectl apply -f /workspace/postgresql-cluster.yaml
check_status "PostgreSQL cluster manifest applied"

# Step 3: Wait for namespace
echo ""
echo -e "${YELLOW}Step 3: Waiting for namespace...${NC}"
sleep 2
kubectl get namespace databases
check_status "Namespace databases exists"

# Step 4: Wait for cluster to be ready
echo ""
echo -e "${YELLOW}Step 4: Waiting for PostgreSQL cluster to be ready...${NC}"
echo "This may take several minutes..."
sleep 10

# Wait for the primary pod
for i in {1..60}; do
    PRIMARY_POD=$(kubectl get pods -n databases -l postgresql=alerthub-pg,role=primary -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    if [ -n "$PRIMARY_POD" ]; then
        echo -e "${GREEN}[OK]${NC} Primary pod found: $PRIMARY_POD"
        break
    fi
    echo "Waiting for primary pod... ($i/60)"
    sleep 5
done

if [ -z "$PRIMARY_POD" ]; then
    echo -e "${RED}[ERROR]${NC} Primary pod not found after 5 minutes"
    echo "Current pods:"
    kubectl get pods -n databases
    exit 1
fi

# Wait for primary pod to be ready
kubectl wait --for=condition=ready pod/"$PRIMARY_POD" -n databases --timeout=300s
check_status "Primary pod is ready"

# Step 5: Apply backup schedules
echo ""
echo -e "${YELLOW}Step 5: Applying backup schedules...${NC}"
kubectl apply -f /workspace/postgresql-backup.yaml
check_status "Backup schedules applied"

# Step 6: Verify cluster status
echo ""
echo -e "${YELLOW}Step 6: Verifying cluster status...${NC}"
kubectl get cluster -n databases alerthub-pg
check_status "Cluster resource exists"

# Step 7: Verify PVCs
echo ""
echo -e "${YELLOW}Step 7: Verifying Persistent Volume Claims...${NC}"
kubectl get pvc -n databases
check_status "PVCs listed"

# Check if PVCs are bound
UNBOUND_PVCS=$(kubectl get pvc -n databases -o jsonpath='{.items[?(@.status.phase!="Bound")].metadata.name}')
if [ -z "$UNBOUND_PVCS" ]; then
    echo -e "${GREEN}[OK]${NC} All PVCs are bound"
else
    echo -e "${YELLOW}[WARNING]${NC} Some PVCs are not bound: $UNBOUND_PVCS"
fi

# Step 8: Verify database exists
echo ""
echo -e "${YELLOW}Step 8: Verifying alerthub database exists...${NC}"
kubectl exec -n databases "$PRIMARY_POD" -- psql -U postgres -c "\l" | grep alerthub
check_status "Database alerthub exists"

# Step 9: Verify database accessibility
echo ""
echo -e "${YELLOW}Step 9: Testing database connectivity...${NC}"
kubectl exec -n databases "$PRIMARY_POD" -- psql -U postgres -d alerthub -c "SELECT version();"
check_status "Database is accessible"

# Step 10: Check cluster health
echo ""
echo -e "${YELLOW}Step 10: Checking cluster health...${NC}"
kubectl get cluster -n databases alerthub-pg -o jsonpath='{.status.phase}'
echo ""
check_status "Cluster status retrieved"

# Step 11: Verify backup configuration
echo ""
echo -e "${YELLOW}Step 11: Verifying backup configuration...${NC}"
kubectl get scheduledbackup -n databases
check_status "Backup schedules listed"

# Summary
echo ""
echo "=========================================="
echo -e "${GREEN}DEPLOYMENT COMPLETED SUCCESSFULLY${NC}"
echo "=========================================="
echo ""
echo "Cluster Details:"
echo "  Name: alerthub-pg"
echo "  Namespace: databases"
echo "  Database: alerthub"
echo "  Instances: 3"
echo ""
echo "Connection Information:"
echo "  Primary Service: alerthub-pg-rw.databases.svc.cluster.local:5432"
echo "  Read-Only Service: alerthub-pg-ro.databases.svc.cluster.local:5432"
echo "  Any Instance: alerthub-pg-r.databases.svc.cluster.local:5432"
echo ""
echo "Get superuser password:"
echo "  kubectl get secret -n databases alerthub-pg-superuser -o jsonpath='{.data.password}' | base64 -d"
echo ""
echo "Access database:"
echo "  kubectl exec -it -n databases $PRIMARY_POD -- psql -U postgres -d alerthub"
echo ""
echo "View cluster status:"
echo "  kubectl get cluster -n databases alerthub-pg"
echo ""
echo "View pods:"
echo "  kubectl get pods -n databases -l postgresql=alerthub-pg"
echo ""
echo "View backups:"
echo "  kubectl get backup -n databases"
echo ""
