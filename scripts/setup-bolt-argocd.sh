#!/bin/bash
# Bolt ArgoCD Setup Script
# Sets up all ArgoCD prerequisites for Bolt dual-mode system

set -e

echo "===== Bolt ArgoCD Setup ====="
echo ""

# Step 1: Update platform project
echo "Step 1: Updating platform project..."
kubectl apply -f infra/gitops/projects/platform-project.yaml
echo "✅ Platform project updated"
echo ""

# Step 2: Deploy cto-apps app-of-apps
echo "Step 2: Deploying cto-apps app-of-apps..."
kubectl apply -f - <<'EOF'
---
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: cto-apps
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: platform
  source:
    repoURL: https://github.com/5dlabs/cto-apps.git
    targetRevision: main
    path: .
    directory:
      recurse: true
      exclude: |
        templates/*
        README.md
        app-of-apps.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=false
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
EOF
echo "✅ cto-apps app-of-apps deployed"
echo ""

# Step 3: Wait for cto-apps to sync
echo "Step 3: Waiting for cto-apps to sync..."
sleep 5
if kubectl wait --for=condition=Synced application/cto-apps -n argocd --timeout=60s 2>/dev/null; then
    echo "✅ cto-apps synced successfully"
else
    echo "⚠️  cto-apps sync pending (this is normal, check status manually)"
fi
echo ""

# Step 4: Verify setup
echo "Step 4: Verifying setup..."
echo ""
echo "Platform Project Status:"
kubectl get appproject platform -n argocd -o jsonpath='{.metadata.name}' && echo " ✅"
echo ""
echo "CTO-Apps Application Status:"
kubectl get application cto-apps -n argocd -o jsonpath='{.metadata.name}{" - "}{.status.sync.status}{" - "}{.status.health.status}' && echo ""
echo ""
echo "Bolt Sensors:"
kubectl get sensors -n argo 2>/dev/null | grep bolt || echo "⚠️  No sensors found (deploy them from cto repo)"
echo ""

echo "===== Setup Complete ====="
echo ""
echo "✅ Platform project updated with:"
echo "   - cto-apps repository whitelisted"
echo "   - Preview namespaces allowed (agent-platform-preview-*)"
echo "   - Production namespaces allowed (agent-platform-prod-*)"
echo ""
echo "✅ CTO-Apps app-of-apps deployed"
echo "   - Watches: https://github.com/5dlabs/cto-apps"
echo "   - Auto-syncs: preview/ and production/ directories"
echo ""
echo "🎯 Next Steps:"
echo "   1. Merge Bolt updates to main branch in cto repo"
echo "   2. ArgoCD will deploy Bolt sensors automatically"
echo "   3. Create a test PR to verify preview deployment"
echo ""
echo "📊 Monitor Activity:"
echo "   - Watch apps: kubectl get applications -n argocd -l managed-by=bolt -w"
echo "   - Watch Git: cd cto-apps && watch git pull"
echo "   - Watch Bolt: kubectl get coderuns -n agent-platform -l agent=bolt -w"
echo ""
echo "Let's fucking go! 🚀"

