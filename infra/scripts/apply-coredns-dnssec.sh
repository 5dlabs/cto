#!/bin/bash

# Script to apply CoreDNS DNSSEC configuration
# This addresses the RFC6840 compliance issue mentioned in CoreDNS issue #5189

set -e

echo "🔧 Applying CoreDNS DNSSEC configuration..."

# Apply the CoreDNS ConfigMap
echo "📝 Applying CoreDNS ConfigMap..."
kubectl apply -f infra/gitops/resources/coredns/coredns-configmap.yaml

# Wait a moment for the ConfigMap to be updated
sleep 2

# Restart CoreDNS pods to pick up the new configuration
echo "🔄 Restarting CoreDNS pods..."
kubectl rollout restart deployment/coredns -n kube-system

# Wait for the rollout to complete
echo "⏳ Waiting for CoreDNS rollout to complete..."
kubectl rollout status deployment/coredns -n kube-system --timeout=120s

# Verify the configuration
echo "✅ Verifying CoreDNS configuration..."
kubectl get configmap coredns -n kube-system -o yaml | grep -A 20 "Corefile:"

echo "🔍 Checking CoreDNS pod status..."
kubectl get pods -n kube-system -l k8s-app=kube-dns

echo "📊 CoreDNS logs (last 10 lines):"
kubectl logs -n kube-system deployment/coredns --tail=10

echo "🎉 CoreDNS DNSSEC configuration applied successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Monitor CoreDNS logs for any DNSSEC validation errors"
echo "2. Test DNS resolution from pods in your cluster"
echo "3. Verify that Mailu can now work with proper DNSSEC validation"
echo ""
echo "🔗 Related issues addressed:"
echo "- CoreDNS issue #5189: https://github.com/coredns/coredns/issues/5189"
echo "- Mailu issue #2239: https://github.com/Mailu/Mailu/issues/2239"
