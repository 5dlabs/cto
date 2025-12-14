#!/bin/bash
# Kind Cluster Setup Script
# Creates a local 2-node Kind cluster for CTO development
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLUSTER_NAME="${CLUSTER_NAME:-cto-dev}"
CONFIG_FILE="${SCRIPT_DIR}/kind-cluster-config.yaml"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     CTO Development Cluster Setup (Kind)                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Check prerequisites
echo ""
echo "🔍 Checking prerequisites..."

if ! command -v kind &> /dev/null; then
    echo "❌ Kind is not installed. Install with:"
    echo "   brew install kind  # macOS"
    echo "   curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.25.0/kind-$(uname)-amd64 && chmod +x ./kind && sudo mv ./kind /usr/local/bin/kind"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed."
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "❌ Docker daemon is not running."
    exit 1
fi

if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed."
    exit 1
fi

if ! command -v helm &> /dev/null; then
    echo "❌ Helm is not installed. Install with:"
    echo "   brew install helm  # macOS"
    exit 1
fi

echo "✅ All prerequisites met"

# Check if cluster already exists
if kind get clusters 2>/dev/null | grep -q "^${CLUSTER_NAME}$"; then
    echo ""
    echo "⚠️  Cluster '${CLUSTER_NAME}' already exists."
    read -p "   Delete and recreate? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🗑️  Deleting existing cluster..."
        kind delete cluster --name "${CLUSTER_NAME}"
    else
        echo "   Using existing cluster."
        kubectl cluster-info --context "kind-${CLUSTER_NAME}" 2>/dev/null || true
        exit 0
    fi
fi

# Create cluster
echo ""
echo "🚀 Creating Kind cluster '${CLUSTER_NAME}'..."
kind create cluster --config "${CONFIG_FILE}" --name "${CLUSTER_NAME}"

# Wait for nodes to be ready
echo ""
echo "⏳ Waiting for nodes to be ready..."
kubectl wait --for=condition=Ready nodes --all --timeout=120s

# Install local-path-provisioner for PVC support
echo ""
echo "📦 Installing local-path-provisioner..."
kubectl apply -f https://raw.githubusercontent.com/rancher/local-path-provisioner/v0.0.28/deploy/local-path-storage.yaml
kubectl patch storageclass local-path -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'

# Create namespaces
echo ""
echo "📁 Creating namespaces..."
kubectl create namespace cto --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace observability --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace automation --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -

# Install ArgoCD
echo ""
echo "🔧 Installing ArgoCD..."
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Wait for ArgoCD to be ready
echo "⏳ Waiting for ArgoCD to be ready..."
kubectl wait --for=condition=Available deployment/argocd-server -n argocd --timeout=300s

# Expose ArgoCD via NodePort
kubectl patch svc argocd-server -n argocd -p '{"spec": {"type": "NodePort", "ports": [{"port": 443, "nodePort": 30080}]}}'

# Get ArgoCD admin password
ARGOCD_PASSWORD=$(kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d)

# Install metrics-server (needed for HPA)
echo ""
echo "📊 Installing metrics-server..."
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
# Patch for Kind (insecure TLS)
kubectl patch deployment metrics-server -n kube-system --type='json' -p='[
  {"op": "add", "path": "/spec/template/spec/containers/0/args/-", "value": "--kubelet-insecure-tls"},
  {"op": "add", "path": "/spec/template/spec/containers/0/args/-", "value": "--kubelet-preferred-address-types=InternalIP"}
]'

# Summary
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     Cluster Setup Complete!                                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Cluster Info:"
kubectl cluster-info --context "kind-${CLUSTER_NAME}"
echo ""
echo "📦 Nodes:"
kubectl get nodes -o wide
echo ""
echo "🔐 ArgoCD Credentials:"
echo "   URL:      https://localhost:30080"
echo "   Username: admin"
echo "   Password: ${ARGOCD_PASSWORD}"
echo ""
echo "🔧 Next Steps:"
echo "   1. Build and load controller image:"
echo "      docker build -t ghcr.io/5dlabs/controller:dev -f infra/images/controller/Dockerfile ."
echo "      kind load docker-image ghcr.io/5dlabs/controller:dev --name ${CLUSTER_NAME}"
echo ""
echo "   2. Install controller with Helm:"
echo "      helm install cto-controller infra/charts/controller \\"
echo "        --namespace cto \\"
echo "        --set image.tag=dev \\"
echo "        --set image.pullPolicy=Never"
echo ""
echo "   3. Or deploy individual components:"
echo "      helm template cto-controller infra/charts/controller | kubectl apply -f -"
echo ""

