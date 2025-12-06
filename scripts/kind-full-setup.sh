#!/bin/bash
# Kind Full Environment Setup
#
# Deploys all components needed for a full play workflow:
# - Argo Workflows
# - Argo Events  
# - Cloudflare Tunnel (dev)
# - Tools Server
# - OpenMemory
# - Secrets from 1Password
# - Webhook EventSources and Sensors
#
# Prerequisites:
#   - Kind cluster running (cto-dev)
#   - All images loaded (kind-local tag)
#   - 1Password CLI authenticated

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_step() { echo -e "\n${BLUE}==>${NC} $1"; }
log_ok() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_err() { echo -e "${RED}✗${NC} $1"; }

# Check prerequisites
check_prereqs() {
    log_step "Checking prerequisites..."
    
    command -v kubectl >/dev/null 2>&1 || { log_err "kubectl not found"; exit 1; }
    command -v helm >/dev/null 2>&1 || { log_err "helm not found"; exit 1; }
    command -v op >/dev/null 2>&1 || { log_err "1Password CLI not found"; exit 1; }
    
    # Check Kind context
    CONTEXT=$(kubectl config current-context)
    if [[ "$CONTEXT" != *"kind"* ]]; then
        log_warn "Context '$CONTEXT' doesn't look like Kind"
        read -p "Continue? (y/N) " -n 1 -r
        echo
        [[ $REPLY =~ ^[Yy]$ ]] || exit 1
    fi
    
    # Check cluster is reachable
    kubectl cluster-info &>/dev/null || { log_err "Cannot reach cluster"; exit 1; }
    
    log_ok "Prerequisites OK"
}

# Create namespaces
setup_namespaces() {
    log_step "Creating namespaces..."
    
    for ns in cto automation argo cloudflare-operator-system; do
        kubectl create namespace $ns --dry-run=client -o yaml | kubectl apply -f -
    done
    
    log_ok "Namespaces ready"
}

# Deploy secrets from 1Password
setup_secrets() {
    log_step "Setting up secrets from 1Password..."
    
    # Run the 1Password secrets script
    if [[ -f "$SCRIPT_DIR/kind-secrets-from-1password.sh" ]]; then
        bash "$SCRIPT_DIR/kind-secrets-from-1password.sh" --all
    else
        log_warn "kind-secrets-from-1password.sh not found, skipping secrets"
    fi
    
    log_ok "Secrets configured"
}

# Install Argo Workflows
install_argo_workflows() {
    log_step "Installing Argo Workflows..."
    
    # Add Argo Helm repo
    helm repo add argo https://argoproj.github.io/argo-helm 2>/dev/null || true
    helm repo update
    
    # Install Argo Workflows
    helm upgrade --install argo-workflows argo/argo-workflows \
        --namespace argo \
        --create-namespace \
        --set server.serviceType=NodePort \
        --set controller.workflowNamespaces="{cto,automation}" \
        --set controller.containerRuntimeExecutor=emissary \
        --wait --timeout 5m
    
    log_ok "Argo Workflows installed"
}

# Install Argo Events
install_argo_events() {
    log_step "Installing Argo Events..."
    
    # Install Argo Events
    helm upgrade --install argo-events argo/argo-events \
        --namespace argo \
        --set controller.replicas=1 \
        --wait --timeout 5m
    
    # Create EventBus
    kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: EventBus
metadata:
  name: default
  namespace: automation
spec:
  nats:
    native:
      replicas: 1
      auth: none
EOF
    
    log_ok "Argo Events installed"
}

# Setup Cloudflare Tunnel
setup_cloudflare_tunnel() {
    log_step "Setting up Cloudflare Tunnel..."
    
    if [[ -f "$SCRIPT_DIR/kind-setup-cloudflare-tunnel.sh" ]]; then
        bash "$SCRIPT_DIR/kind-setup-cloudflare-tunnel.sh"
    else
        log_warn "Cloudflare tunnel script not found, skipping"
    fi
    
    log_ok "Cloudflare Tunnel configured"
}

# Deploy Tools Server
deploy_tools_server() {
    log_step "Deploying Tools Server..."
    
    kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tools-server
  namespace: cto
spec:
  replicas: 1
  selector:
    matchLabels:
      app: tools-server
  template:
    metadata:
      labels:
        app: tools-server
    spec:
      containers:
      - name: tools-server
        image: ghcr.io/5dlabs/tools:kind-local
        imagePullPolicy: Never
        ports:
        - containerPort: 3000
        env:
        - name: MCP_PORT
          value: "3000"
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: tools-server
  namespace: cto
spec:
  selector:
    app: tools-server
  ports:
  - port: 3000
    targetPort: 3000
EOF
    
    log_ok "Tools Server deployed"
}

# Deploy OpenMemory
deploy_openmemory() {
    log_step "Deploying OpenMemory..."
    
    kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: openmemory
  namespace: cto
spec:
  replicas: 1
  selector:
    matchLabels:
      app: openmemory
  template:
    metadata:
      labels:
        app: openmemory
    spec:
      containers:
      - name: openmemory
        image: ghcr.io/5dlabs/openmemory:kind-local
        imagePullPolicy: Never
        ports:
        - containerPort: 8080
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: openmemory
  namespace: cto
spec:
  selector:
    app: openmemory
  ports:
  - port: 8080
    targetPort: 8080
EOF
    
    log_ok "OpenMemory deployed"
}

# Setup GitHub EventSource and Sensors
setup_github_eventsource() {
    log_step "Setting up GitHub EventSource..."
    
    # Create EventSource for GitHub webhooks
    kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: github-eventsource
  namespace: automation
spec:
  service:
    ports:
    - port: 12000
      targetPort: 12000
  github:
    cto-webhook:
      repositories:
      - owner: 5dlabs
        names:
        - agent-sandbox
        - cto
      webhook:
        endpoint: /github/webhook
        port: "12000"
        method: POST
      contentType: json
      insecure: false
      active: true
      events:
      - push
      - pull_request
      - issue_comment
      - pull_request_review
      apiToken:
        name: github-webhook-secret
        key: token
      webhookSecret:
        name: github-webhook-secret
        key: secret
EOF
    
    log_ok "GitHub EventSource created"
}

# Create TunnelBinding for webhooks
setup_tunnel_binding() {
    log_step "Setting up TunnelBinding for webhooks..."
    
    kubectl apply -f - <<EOF
apiVersion: networking.cfargotunnel.com/v1alpha1
kind: TunnelBinding
metadata:
  name: github-webhooks-dev
  namespace: automation
tunnelRef:
  kind: ClusterTunnel
  name: cto-dev
subjects:
- name: github-eventsource-svc
  spec:
    fqdn: github-webhooks-dev.5dlabs.ai
    path: /github/webhook
    target: http://github-eventsource-svc.automation.svc:12000
EOF
    
    log_ok "TunnelBinding created"
}

# Print status
print_status() {
    log_step "Environment Status"
    
    echo -e "\n${GREEN}=== Deployments ===${NC}"
    kubectl get deployments -A | grep -E "NAME|cto|argo|automation"
    
    echo -e "\n${GREEN}=== Services ===${NC}"
    kubectl get svc -A | grep -E "NAME|cto|argo|automation"
    
    echo -e "\n${GREEN}=== EventSources ===${NC}"
    kubectl get eventsources -n automation 2>/dev/null || echo "No EventSources yet"
    
    echo -e "\n${GREEN}=== Tunnels ===${NC}"
    kubectl get clustertunnels 2>/dev/null || echo "No tunnels yet"
    
    echo -e "\n${BLUE}=== Next Steps ===${NC}"
    echo "1. Update GitHub App webhook URLs to: https://github-webhooks-dev.5dlabs.ai/github/webhook"
    echo "2. Install GitHub Apps on 5dlabs/agent-sandbox"
    echo "3. Test with: kubectl apply -f <coderun.yaml>"
}

# Main
main() {
    echo -e "${GREEN}"
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║     Kind Full Environment Setup for CTO Platform      ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    check_prereqs
    setup_namespaces
    setup_secrets
    install_argo_workflows
    install_argo_events
    setup_cloudflare_tunnel
    deploy_tools_server
    deploy_openmemory
    setup_github_eventsource
    setup_tunnel_binding
    print_status
    
    echo -e "\n${GREEN}✓ Setup complete!${NC}"
}

# Run
main "$@"



