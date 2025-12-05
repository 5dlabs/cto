#!/bin/bash
# Log snapshot script - captures logs from all pods in cto namespace
# Runs continuously every 2 seconds

set -euo pipefail

LOGS_DIR="/Users/jonathonfritz/test/cto/logs"
NAMESPACE="cto"

mkdir -p "$LOGS_DIR"

echo "Starting log snapshot service (every 2 seconds)..."
echo "Logs directory: $LOGS_DIR"
echo "Namespace: $NAMESPACE"
echo "Press Ctrl+C to stop"
echo ""

while true; do
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  
  # Get all pods in namespace
  pods=$(kubectl get pods -n "$NAMESPACE" -o jsonpath='{range .items[*]}{.metadata.name}{"\n"}{end}' 2>/dev/null || echo "")
  
  if [ -z "$pods" ]; then
    echo "[$timestamp] No pods found or kubectl error"
  else
    pod_count=0
    for pod in $pods; do
      kubectl logs -n "$NAMESPACE" "$pod" --all-containers=true > "${LOGS_DIR}/${pod}.log" 2>&1 || true
      ((pod_count++)) || true
    done
    echo "[$timestamp] Captured logs from $pod_count pods"
  fi
  
  sleep 2
done












