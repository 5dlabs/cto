#!/bin/bash
# Install database operators on Kind cluster
# All operators are Apache 2.0 licensed - safe to distribute
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NAMESPACE_DATABASES="databases"

echo "============================================"
echo "  Database Operators Installation Script"
echo "  License: Apache 2.0 (All operators)"
echo "============================================"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found. Please install kubectl."
        exit 1
    fi
    
    if ! command -v helm &> /dev/null; then
        log_error "helm not found. Please install helm."
        exit 1
    fi
    
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster."
        exit 1
    fi
    
    log_info "Prerequisites check passed!"
}

# Create databases namespace
create_namespace() {
    log_info "Creating databases namespace..."
    kubectl create namespace "$NAMESPACE_DATABASES" --dry-run=client -o yaml | kubectl apply -f -
}

# Install Strimzi Kafka Operator
install_strimzi() {
    log_info "Installing Strimzi Kafka Operator..."
    
    # Add Strimzi Helm repo
    helm repo add strimzi https://strimzi.io/charts/ 2>/dev/null || true
    helm repo update
    
    # Install operator
    helm upgrade --install strimzi-kafka-operator strimzi/strimzi-kafka-operator \
        --namespace strimzi \
        --create-namespace \
        --version 0.49.0 \
        --set watchAnyNamespace=true \
        --set resources.requests.cpu=100m \
        --set resources.requests.memory=256Mi \
        --set resources.limits.cpu=500m \
        --set resources.limits.memory=512Mi \
        --wait --timeout 5m
    
    log_info "Strimzi Kafka Operator installed!"
}

# Install Altinity ClickHouse Operator
install_clickhouse() {
    log_info "Installing Altinity ClickHouse Operator..."
    
    # Install from Altinity's official manifests (deploys to kube-system)
    kubectl apply -f https://raw.githubusercontent.com/Altinity/clickhouse-operator/release-0.25.5/deploy/operator/clickhouse-operator-install-bundle.yaml
    
    # Wait for operator to be ready
    log_info "Waiting for ClickHouse operator to be ready..."
    kubectl wait --for=condition=available deployment/clickhouse-operator \
        -n kube-system --timeout=300s || true
    
    log_info "Altinity ClickHouse Operator installed!"
}

# Install OpenSearch Operator
install_opensearch() {
    log_info "Installing OpenSearch Kubernetes Operator..."
    
    # Add OpenSearch Helm repo
    helm repo add opensearch-operator https://opensearch-project.github.io/opensearch-k8s-operator/ 2>/dev/null || true
    helm repo update
    
    # Install operator
    helm upgrade --install opensearch-operator opensearch-operator/opensearch-operator \
        --namespace opensearch-operator \
        --create-namespace \
        --version 2.8.0 \
        --set manager.watchNamespace="" \
        --set manager.resources.requests.cpu=100m \
        --set manager.resources.requests.memory=256Mi \
        --set manager.resources.limits.cpu=500m \
        --set manager.resources.limits.memory=512Mi \
        --wait --timeout 5m
    
    log_info "OpenSearch Operator installed!"
}

# Create test instances
create_test_instances() {
    log_info "Creating test database instances..."
    
    # Apply all test instance manifests
    kubectl apply -f "$SCRIPT_DIR/test-instances/" -n "$NAMESPACE_DATABASES" || true
    
    log_info "Test instances created!"
}

# Wait for pods to be ready
wait_for_pods() {
    log_info "Waiting for operator pods to be ready..."
    
    # Strimzi
    kubectl wait --for=condition=ready pod -l name=strimzi-cluster-operator \
        -n strimzi --timeout=300s 2>/dev/null || log_warn "Strimzi pods not ready yet"
    
    # ClickHouse (deploys to kube-system)
    kubectl wait --for=condition=ready pod -l app=clickhouse-operator \
        -n kube-system --timeout=300s 2>/dev/null || log_warn "ClickHouse operator pods not ready yet"
    
    # OpenSearch
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=opensearch-operator \
        -n opensearch-operator --timeout=300s 2>/dev/null || log_warn "OpenSearch operator pods not ready yet"
}

# Print status
print_status() {
    echo ""
    echo "============================================"
    echo "  Installation Status"
    echo "============================================"
    echo ""
    
    log_info "Operator Pods:"
    kubectl get pods -n strimzi 2>/dev/null || echo "  Strimzi namespace not found"
    kubectl get pods -n kube-system -l app=clickhouse-operator 2>/dev/null || echo "  ClickHouse operator not found"
    kubectl get pods -n opensearch-operator 2>/dev/null || echo "  OpenSearch namespace not found"
    
    echo ""
    log_info "Database CRDs:"
    kubectl get crd | grep -E "(kafka|clickhouse|opensearch)" || echo "  No database CRDs found"
    
    echo ""
    log_info "Test Instances (databases namespace):"
    kubectl get kafka,clickhouseinstallation,opensearchcluster -n "$NAMESPACE_DATABASES" 2>/dev/null || echo "  No test instances yet"
}

# Main
main() {
    case "${1:-all}" in
        strimzi)
            check_prerequisites
            create_namespace
            install_strimzi
            wait_for_pods
            ;;
        clickhouse)
            check_prerequisites
            create_namespace
            install_clickhouse
            wait_for_pods
            ;;
        opensearch)
            check_prerequisites
            create_namespace
            install_opensearch
            wait_for_pods
            ;;
        instances)
            create_test_instances
            ;;
        status)
            print_status
            ;;
        all)
            check_prerequisites
            create_namespace
            install_strimzi
            install_clickhouse
            install_opensearch
            wait_for_pods
            print_status
            ;;
        *)
            echo "Usage: $0 {strimzi|clickhouse|opensearch|instances|status|all}"
            exit 1
            ;;
    esac
}

main "$@"

