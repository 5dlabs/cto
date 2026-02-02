#!/bin/bash
set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

NAMESPACE="messaging"
CLUSTER_NAME="kafka-cluster"

echo -e "${GREEN}=== Kafka Cluster Deployment Script ===${NC}"
echo ""

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}ERROR: kubectl is not installed${NC}"
    exit 1
fi

# Check if Strimzi operator is installed
echo -e "${YELLOW}Checking for Strimzi operator...${NC}"
if ! kubectl get crd kafkas.kafka.strimzi.io &> /dev/null; then
    echo -e "${YELLOW}Strimzi operator not found. Installing...${NC}"
    kubectl create namespace kafka || true
    kubectl create -f 'https://strimzi.io/install/latest?namespace=kafka' -n kafka

    echo -e "${YELLOW}Waiting for Strimzi operator to be ready...${NC}"
    kubectl wait --for=condition=ready pod -l name=strimzi-cluster-operator -n kafka --timeout=300s
    echo -e "${GREEN}Strimzi operator installed successfully${NC}"
else
    echo -e "${GREEN}Strimzi operator already installed${NC}"
fi

echo ""
echo -e "${YELLOW}Deploying Kafka cluster...${NC}"

# Create namespace
kubectl apply -f namespace.yaml

# Deploy metrics configuration
kubectl apply -f metrics-config.yaml

# Deploy Kafka cluster
kubectl apply -f cluster.yaml

# Wait for Kafka cluster to be ready
echo -e "${YELLOW}Waiting for Kafka cluster to be ready (this may take several minutes)...${NC}"
kubectl wait kafka/${CLUSTER_NAME} --for=condition=Ready --timeout=600s -n ${NAMESPACE}

# Deploy topics
echo -e "${YELLOW}Creating Kafka topics...${NC}"
kubectl apply -f topics.yaml

# Deploy users
echo -e "${YELLOW}Creating Kafka users...${NC}"
kubectl apply -f users.yaml

# Deploy pod disruption budgets
echo -e "${YELLOW}Applying pod disruption budgets...${NC}"
kubectl apply -f pod-disruption-budget.yaml

# Deploy network policy
echo -e "${YELLOW}Applying network policy...${NC}"
kubectl apply -f network-policy.yaml

echo ""
echo -e "${GREEN}=== Deployment Complete ===${NC}"
echo ""
echo -e "Cluster Status:"
kubectl get kafka ${CLUSTER_NAME} -n ${NAMESPACE}

echo ""
echo -e "Pods:"
kubectl get pods -n ${NAMESPACE} -l strimzi.io/cluster=${CLUSTER_NAME}

echo ""
echo -e "Topics:"
kubectl get kafkatopics -n ${NAMESPACE}

echo ""
echo -e "Users:"
kubectl get kafkausers -n ${NAMESPACE}

echo ""
echo -e "${GREEN}Bootstrap servers:${NC}"
echo -e "  Plain: ${CLUSTER_NAME}-kafka-bootstrap.${NAMESPACE}.svc.cluster.local:9092"
echo -e "  TLS:   ${CLUSTER_NAME}-kafka-bootstrap.${NAMESPACE}.svc.cluster.local:9093"

echo ""
echo -e "${YELLOW}To monitor the deployment, run:${NC}"
echo -e "  kubectl get pods -n ${NAMESPACE} -w"
echo ""
echo -e "${YELLOW}To view logs, run:${NC}"
echo -e "  kubectl logs -n ${NAMESPACE} ${CLUSTER_NAME}-kafka-0 -c kafka"
echo ""
