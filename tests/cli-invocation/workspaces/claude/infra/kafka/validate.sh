#!/bin/bash

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

NAMESPACE="messaging"
CLUSTER_NAME="kafka-cluster"

echo -e "${GREEN}=== Kafka Cluster Validation ===${NC}"
echo ""

# Function to check status
check_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
        return 0
    else
        echo -e "${RED}✗ $2${NC}"
        return 1
    fi
}

ERRORS=0

# Check namespace exists
echo -e "${YELLOW}Checking namespace...${NC}"
kubectl get namespace ${NAMESPACE} &> /dev/null
check_status $? "Namespace '${NAMESPACE}' exists" || ((ERRORS++))

# Check Kafka cluster
echo -e "${YELLOW}Checking Kafka cluster...${NC}"
kubectl get kafka ${CLUSTER_NAME} -n ${NAMESPACE} &> /dev/null
check_status $? "Kafka cluster '${CLUSTER_NAME}' exists" || ((ERRORS++))

# Check Kafka cluster status
KAFKA_READY=$(kubectl get kafka ${CLUSTER_NAME} -n ${NAMESPACE} -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null)
if [ "$KAFKA_READY" == "True" ]; then
    check_status 0 "Kafka cluster is Ready"
else
    check_status 1 "Kafka cluster is NOT Ready (Status: ${KAFKA_READY})"
    ((ERRORS++))
fi

# Check Kafka pods
echo -e "${YELLOW}Checking Kafka pods...${NC}"
KAFKA_REPLICAS=3
KAFKA_READY_PODS=$(kubectl get pods -n ${NAMESPACE} -l strimzi.io/name=${CLUSTER_NAME}-kafka --no-headers 2>/dev/null | grep -c "Running")
if [ "$KAFKA_READY_PODS" -eq "$KAFKA_REPLICAS" ]; then
    check_status 0 "All $KAFKA_REPLICAS Kafka brokers are running"
else
    check_status 1 "Expected $KAFKA_REPLICAS Kafka brokers, found $KAFKA_READY_PODS running"
    ((ERRORS++))
fi

# Check ZooKeeper pods
echo -e "${YELLOW}Checking ZooKeeper pods...${NC}"
ZK_REPLICAS=3
ZK_READY_PODS=$(kubectl get pods -n ${NAMESPACE} -l strimzi.io/name=${CLUSTER_NAME}-zookeeper --no-headers 2>/dev/null | grep -c "Running")
if [ "$ZK_READY_PODS" -eq "$ZK_REPLICAS" ]; then
    check_status 0 "All $ZK_REPLICAS ZooKeeper nodes are running"
else
    check_status 1 "Expected $ZK_REPLICAS ZooKeeper nodes, found $ZK_READY_PODS running"
    ((ERRORS++))
fi

# Check Entity Operator
echo -e "${YELLOW}Checking Entity Operator...${NC}"
kubectl get deployment ${CLUSTER_NAME}-entity-operator -n ${NAMESPACE} &> /dev/null
check_status $? "Entity Operator deployment exists" || ((ERRORS++))

EO_READY=$(kubectl get deployment ${CLUSTER_NAME}-entity-operator -n ${NAMESPACE} -o jsonpath='{.status.readyReplicas}' 2>/dev/null)
if [ "$EO_READY" == "1" ]; then
    check_status 0 "Entity Operator is ready"
else
    check_status 1 "Entity Operator is not ready"
    ((ERRORS++))
fi

# Check topics
echo -e "${YELLOW}Checking Kafka topics...${NC}"
EXPECTED_TOPICS=("events" "commands" "logs")
for topic in "${EXPECTED_TOPICS[@]}"; do
    kubectl get kafkatopic ${topic} -n ${NAMESPACE} &> /dev/null
    check_status $? "Topic '${topic}' exists" || ((ERRORS++))
done

# Check users
echo -e "${YELLOW}Checking Kafka users...${NC}"
EXPECTED_USERS=("kafka-producer" "kafka-consumer" "kafka-admin")
for user in "${EXPECTED_USERS[@]}"; do
    kubectl get kafkauser ${user} -n ${NAMESPACE} &> /dev/null
    check_status $? "User '${user}' exists" || ((ERRORS++))
done

# Check services
echo -e "${YELLOW}Checking services...${NC}"
kubectl get service ${CLUSTER_NAME}-kafka-bootstrap -n ${NAMESPACE} &> /dev/null
check_status $? "Kafka bootstrap service exists" || ((ERRORS++))

kubectl get service ${CLUSTER_NAME}-zookeeper-client -n ${NAMESPACE} &> /dev/null
check_status $? "ZooKeeper client service exists" || ((ERRORS++))

# Check persistent volumes
echo -e "${YELLOW}Checking persistent volumes...${NC}"
KAFKA_PVCS=$(kubectl get pvc -n ${NAMESPACE} -l strimzi.io/name=${CLUSTER_NAME}-kafka --no-headers 2>/dev/null | grep -c "Bound")
if [ "$KAFKA_PVCS" -eq "$KAFKA_REPLICAS" ]; then
    check_status 0 "All Kafka PVCs are bound"
else
    check_status 1 "Expected $KAFKA_REPLICAS Kafka PVCs, found $KAFKA_PVCS bound"
    ((ERRORS++))
fi

ZK_PVCS=$(kubectl get pvc -n ${NAMESPACE} -l strimzi.io/name=${CLUSTER_NAME}-zookeeper --no-headers 2>/dev/null | grep -c "Bound")
if [ "$ZK_PVCS" -eq "$ZK_REPLICAS" ]; then
    check_status 0 "All ZooKeeper PVCs are bound"
else
    check_status 1 "Expected $ZK_REPLICAS ZooKeeper PVCs, found $ZK_PVCS bound"
    ((ERRORS++))
fi

# Check pod disruption budgets
echo -e "${YELLOW}Checking pod disruption budgets...${NC}"
kubectl get pdb ${CLUSTER_NAME}-kafka-pdb -n ${NAMESPACE} &> /dev/null
check_status $? "Kafka PodDisruptionBudget exists" || ((ERRORS++))

kubectl get pdb ${CLUSTER_NAME}-zookeeper-pdb -n ${NAMESPACE} &> /dev/null
check_status $? "ZooKeeper PodDisruptionBudget exists" || ((ERRORS++))

# Summary
echo ""
echo -e "${GREEN}=== Validation Summary ===${NC}"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}All checks passed! ✓${NC}"
    echo ""
    echo -e "${GREEN}Cluster Information:${NC}"
    echo -e "  Bootstrap servers (Plain): ${CLUSTER_NAME}-kafka-bootstrap.${NAMESPACE}.svc.cluster.local:9092"
    echo -e "  Bootstrap servers (TLS):   ${CLUSTER_NAME}-kafka-bootstrap.${NAMESPACE}.svc.cluster.local:9093"
    echo ""
    echo -e "${GREEN}Next steps:${NC}"
    echo -e "  1. Test connectivity to the cluster"
    echo -e "  2. Extract user certificates for authentication"
    echo -e "  3. Configure your applications to use the bootstrap servers"
    exit 0
else
    echo -e "${RED}Validation failed with $ERRORS error(s)${NC}"
    echo ""
    echo -e "${YELLOW}Troubleshooting commands:${NC}"
    echo -e "  kubectl get pods -n ${NAMESPACE}"
    echo -e "  kubectl describe kafka ${CLUSTER_NAME} -n ${NAMESPACE}"
    echo -e "  kubectl logs -n ${NAMESPACE} ${CLUSTER_NAME}-kafka-0 -c kafka"
    echo -e "  kubectl get events -n ${NAMESPACE} --sort-by='.lastTimestamp'"
    exit 1
fi
