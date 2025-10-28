#!/bin/bash

# Debug script to investigate DocsRun issues

echo "=== DocsRun Resources ==="
kubectl get docsrun -A

echo -e "\n=== DocsRun Details ==="
kubectl get docsrun -A -o yaml

echo -e "\n=== Jobs Related to Docs ==="
kubectl get jobs -A | grep docs

echo -e "\n=== Job Details ==="
kubectl get jobs -A -o yaml | grep -A 20 -B 5 docs

echo -e "\n=== PVCs Related to Docs ==="
kubectl get pvc -A | grep docs

echo -e "\n=== Pods Related to Docs ==="
kubectl get pods -A | grep docs

echo -e "\n=== ConfigMaps Related to Docs ==="
kubectl get configmaps -A | grep docs

echo -e "\n=== Controller Logs (last 50 lines) ==="
kubectl logs -n agent-platform deployment/agent-controller --tail=50

echo -e "\n=== Workflow Status ==="
kubectl get workflows -A | grep docsrun
