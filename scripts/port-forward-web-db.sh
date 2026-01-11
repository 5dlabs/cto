#!/bin/bash
# Port-forward helper for web app database local development
# Forwards PostgreSQL service from databases namespace to localhost:5432

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SERVICE_NAME="web-postgres-rw"
NAMESPACE="databases"
LOCAL_PORT="${LOCAL_PORT:-5432}"

echo -e "${BLUE}🔌 Port-forwarding PostgreSQL for web app development${NC}"
echo ""
echo -e "Service: ${GREEN}${SERVICE_NAME}${NC}"
echo -e "Namespace: ${GREEN}${NAMESPACE}${NC}"
echo -e "Local port: ${GREEN}${LOCAL_PORT}${NC}"
echo ""

# Check if port is already in use
if lsof -Pi :${LOCAL_PORT} -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo -e "${YELLOW}⚠️  Port ${LOCAL_PORT} is already in use${NC}"
    echo -e "${YELLOW}   Killing existing process...${NC}"
    lsof -ti:${LOCAL_PORT} | xargs kill -9 2>/dev/null || true
    sleep 1
fi

# Get database connection details
echo -e "${BLUE}📋 Database connection details:${NC}"
SECRET_EXISTS=$(kubectl get secret web-postgres-app -n ${NAMESPACE} 2>/dev/null || echo "")
if [ -n "$SECRET_EXISTS" ]; then
    DB_USER=$(kubectl get secret web-postgres-app -n ${NAMESPACE} -o jsonpath='{.data.username}' | base64 -d)
    DB_NAME=$(kubectl get secret web-postgres-app -n ${NAMESPACE} -o jsonpath='{.data.dbname}' | base64 -d)
    DB_PASSWORD=$(kubectl get secret web-postgres-app -n ${NAMESPACE} -o jsonpath='{.data.password}' | base64 -d)
    
    echo -e "  Database: ${GREEN}${DB_NAME}${NC}"
    echo -e "  Username: ${GREEN}${DB_USER}${NC}"
    echo -e "  Password: ${GREEN}${DB_PASSWORD:0:8}...${NC} (first 8 chars)"
    echo ""
    echo -e "${BLUE}💡 Connection string for local development:${NC}"
    echo -e "${GREEN}postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${LOCAL_PORT}/${DB_NAME}${NC}"
    echo ""
else
    echo -e "${YELLOW}⚠️  Secret web-postgres-app not found in ${NAMESPACE} namespace${NC}"
    echo -e "${YELLOW}   The PostgreSQL cluster may not be ready yet.${NC}"
    echo -e "${YELLOW}   Wait for the cluster to be created and try again.${NC}"
    echo ""
fi

# Start port-forward
echo -e "${BLUE}🚀 Starting port-forward...${NC}"
echo -e "${BLUE}   Press Ctrl+C to stop${NC}"
echo ""

kubectl port-forward svc/${SERVICE_NAME} -n ${NAMESPACE} ${LOCAL_PORT}:5432
