#!/usr/bin/env bash
# Diagnose why OpenMemory and/or tools server pods are not running in cto namespace.
# Usage: ./scripts/2026-02/diagnose-cto-pods.sh

set -euo pipefail

NAMESPACE="${NAMESPACE:-cto}"

echo "=== CTO namespace pods diagnostic ==="
echo "Namespace: $NAMESPACE"
echo ""

echo "--- Argo CD application (cto) ---"
if command -v argocd &>/dev/null; then
  argocd app get cto -n argocd 2>/dev/null || echo "argocd app get failed (not logged in or app missing)"
else
  echo "argocd CLI not installed; skip or run: kubectl get application cto -n argocd -o yaml"
fi
echo ""

echo "--- All pods in $NAMESPACE ---"
kubectl get pods -n "$NAMESPACE" -o wide 2>/dev/null || { echo "kubectl failed (context?)"; exit 1; }
echo ""

echo "--- OpenMemory pods ---"
kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=openmemory -o wide 2>/dev/null || true
echo ""

echo "--- Tools server pods ---"
kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/component=tools -o wide 2>/dev/null || true
echo ""

echo "--- PVCs in $NAMESPACE (OpenMemory/tools use mayastor) ---"
kubectl get pvc -n "$NAMESPACE" 2>/dev/null || true
echo ""

echo "--- Recent events in $NAMESPACE ---"
kubectl get events -n "$NAMESPACE" --sort-by='.lastTimestamp' 2>/dev/null | tail -20 || true
echo ""

echo "--- OpenMemory deployment (if any) ---"
kubectl get deployment -n "$NAMESPACE" -l app.kubernetes.io/name=openmemory -o wide 2>/dev/null || true
echo ""

echo "--- Tools deployment (if any) ---"
kubectl get deployment -n "$NAMESPACE" -l app.kubernetes.io/component=tools -o wide 2>/dev/null || true
echo ""

echo "Done. For pod logs: kubectl logs -n $NAMESPACE -l app.kubernetes.io/name=openmemory --tail=50"
echo "For tools: kubectl logs -n $NAMESPACE -l app.kubernetes.io/component=tools -c tools --tail=50"
