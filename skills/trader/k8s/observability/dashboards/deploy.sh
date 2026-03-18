#!/usr/bin/env bash
# Deploy observability stack to Cherry trading cluster
# Usage: KUBECONFIG=/tmp/trading-talos/kubeconfig ./deploy.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OBS_DIR="$(dirname "$SCRIPT_DIR")"
K8S_DIR="$(dirname "$OBS_DIR")"
KUBECONFIG="${KUBECONFIG:-/tmp/trading-talos/kubeconfig}"

export KUBECONFIG

# Announce step via macOS say (non-blocking)
announce() { say "$1" & }

announce "Starting observability deploy"
echo "=== Cherry Trading Cluster: Observability Stack Deploy ==="
echo "Using KUBECONFIG: $KUBECONFIG"
echo ""

# 1. Namespace + PodSecurity labels
announce "Step 1: Creating namespace"
echo "[1/12] Creating observability namespace..."
kubectl create namespace observability --dry-run=client -o yaml | kubectl apply -f -
kubectl label namespace observability pod-security.kubernetes.io/enforce=privileged --overwrite

# 2. local-path-provisioner (skip if already present)
announce "Step 2: Local path provisioner"
echo "[2/12] Checking local-path-provisioner..."
if kubectl get sc local-path &>/dev/null; then
  echo "  ✓ local-path StorageClass already exists, skipping install"
else
  echo "  Installing local-path-provisioner..."
  kubectl apply -f https://raw.githubusercontent.com/rancher/local-path-provisioner/v0.0.31/deploy/local-path-provisioner.yaml
fi

# 3. Discord webhook secret check
announce "Step 3: Checking Discord secret"
echo "[3/12] Checking Discord webhook secret..."
if kubectl -n observability get secret alertmanager-discord-secret &>/dev/null; then
  echo "  ✓ alertmanager-discord-secret exists"
else
  echo "  ⚠ alertmanager-discord-secret NOT FOUND"
  echo "  AlertManager and Grafana alerts will not deliver to Discord."
  echo "  Create it with:"
  echo "    kubectl -n observability create secret generic alertmanager-discord-secret \\"
  echo "      --from-literal=DISCORD_WEBHOOK_URL='https://discord.com/api/webhooks/<ID>/<TOKEN>'"
  echo ""
  read -rp "  Continue without Discord alerts? [y/N] " answer
  if [[ ! "$answer" =~ ^[Yy]$ ]]; then
    announce "Deploy aborted. Discord secret missing."
    echo "Aborting. Create the secret and re-run."
    exit 1
  fi
fi

# 4. Loki (must be up before Fluent Bit)
announce "Step 4: Deploying Loki"
echo "[4/12] Deploying Loki..."
helm repo add grafana https://grafana.github.io/helm-charts 2>/dev/null || true
helm repo update grafana
helm upgrade --install loki grafana/loki \
  --namespace observability \
  --version 6.16.0 \
  -f "$OBS_DIR/loki-values.yaml" \
  --force-conflicts --wait --timeout 5m
announce "Loki deployed"

# 5. Fluent Bit
announce "Step 5: Deploying Fluent Bit"
echo "[5/12] Deploying Fluent Bit..."
helm repo add fluent https://fluent.github.io/helm-charts 2>/dev/null || true
helm repo update fluent
helm upgrade --install fluent-bit fluent/fluent-bit \
  --namespace observability \
  --version 0.47.7 \
  -f "$OBS_DIR/fluent-bit-values.yaml" \
  --force-conflicts --wait --timeout 3m
announce "Fluent Bit deployed"

# 6. Prometheus + AlertManager + node-exporter + kube-state-metrics
announce "Step 6: Deploying Prometheus and Alert Manager"
echo "[6/12] Deploying Prometheus + AlertManager..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts 2>/dev/null || true
helm repo update prometheus-community
helm upgrade --install prometheus prometheus-community/prometheus \
  --namespace observability \
  --version 25.27.0 \
  -f "$OBS_DIR/prometheus-values.yaml" \
  --force-conflicts --wait --timeout 5m
announce "Prometheus and Alert Manager deployed"

# 7. Cilium upgrade (add prometheus + hubble.metrics)
announce "Step 7: Upgrading Cilium"
echo "[7/12] Upgrading Cilium with metrics..."
helm repo add cilium https://helm.cilium.io/ 2>/dev/null || true
helm repo update cilium
helm upgrade cilium cilium/cilium \
  --namespace kube-system \
  -f /tmp/trading-talos/cilium-values.yaml \
  --force-conflicts --wait --timeout 5m
announce "Cilium upgraded"

# 8. Grafana (includes AlertManager datasource + log-based alert rules)
announce "Step 8: Deploying Grafana with alerting"
echo "[8/12] Deploying Grafana..."
helm upgrade --install grafana grafana/grafana \
  --namespace observability \
  --version 9.2.9 \
  -f "$OBS_DIR/grafana-values.yaml" \
  --force-conflicts --wait --timeout 5m
announce "Grafana deployed"

# 9. Dashboard ConfigMaps
announce "Step 9: Applying dashboards"
echo "[9/12] Applying dashboard ConfigMaps..."
kubectl apply -f "$SCRIPT_DIR/"

# 10. Agave RPC (sidecar pattern — triggers Recreate rollout)
announce "Step 10: Deploying Agave with sidecar pattern. This triggers a full restart."
echo "[10/12] Updating Agave RPC deployment (sidecar pattern)..."
kubectl apply -f "$K8S_DIR/agave-rpc.yaml"

# 11. AlertManager verification
announce "Step 11: Verifying Alert Manager"
echo "[11/12] Verifying AlertManager..."
echo "  Waiting for AlertManager pod..."
kubectl -n observability wait --for=condition=ready pod -l app.kubernetes.io/name=alertmanager --timeout=120s 2>/dev/null || \
  kubectl -n observability wait --for=condition=ready pod -l app=prometheus,component=alertmanager --timeout=120s 2>/dev/null || \
  echo "  ⚠ AlertManager pod not ready yet (may still be starting)"

AM_POD=$(kubectl -n observability get pods -l "component=alertmanager" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
if [ -n "$AM_POD" ]; then
  echo "  ✓ AlertManager pod: $AM_POD"
  announce "Alert Manager is running"
else
  echo "  ⚠ Could not find AlertManager pod (check: kubectl -n observability get pods)"
  announce "Warning: Alert Manager pod not found"
fi

# 12. Post-deploy validation
announce "Step 12: Running post deploy validation"
echo "[12/12] Post-deploy validation..."
echo ""
echo "--- Pod Status ---"
kubectl -n observability get pods -o wide
echo ""
kubectl -n solana get pods -o wide
echo ""

echo "--- Prometheus Targets ---"
PROM_POD=$(kubectl -n observability get pods -l "app.kubernetes.io/name=prometheus,app.kubernetes.io/component=server" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
if [ -n "$PROM_POD" ]; then
  echo "  Checking targets via port-forward (5s)..."
  kubectl -n observability port-forward "$PROM_POD" 9090:9090 &>/dev/null &
  PF_PID=$!
  sleep 2
  TARGETS_UP=$(curl -s http://localhost:9090/api/v1/targets 2>/dev/null | grep -o '"health":"up"' | wc -l || echo "0")
  TARGETS_DOWN=$(curl -s http://localhost:9090/api/v1/targets 2>/dev/null | grep -o '"health":"down"' | wc -l || echo "0")
  RULES=$(curl -s http://localhost:9090/api/v1/rules 2>/dev/null | grep -o '"type":"alerting"' | wc -l || echo "0")
  kill $PF_PID 2>/dev/null || true
  wait $PF_PID 2>/dev/null || true
  echo "  Targets UP: $TARGETS_UP | DOWN: $TARGETS_DOWN"
  echo "  Alert rules loaded: $RULES"
else
  echo "  ⚠ Could not find Prometheus server pod for validation"
fi

echo ""
echo "=== Deploy Complete ==="
announce "Deploy complete. All 12 steps finished. Check Grafana for dashboards."
echo ""
echo "Verification commands:"
echo "  kubectl -n solana logs deployment/agave-rpc -c wait-for-rpc --tail=5"
echo "  kubectl -n solana logs deployment/agave-rpc -c agave --tail=5"
echo "  kubectl -n observability get pods"
echo "  curl http://84.32.176.180:30300  (Grafana)"
echo ""
echo "Grafana: admin/admin | Dashboards: Trading Cluster folder"
echo "AlertManager: kubectl -n observability port-forward svc/prometheus-alertmanager 9093:9093"
echo ""
echo "NOTE: Agave takes 10-30+ min to load snapshot. Solana exporter will"
echo "      start only after RPC is ready (wait-for-rpc init container)."
