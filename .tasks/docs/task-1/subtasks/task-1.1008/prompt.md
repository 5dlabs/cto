Implement subtask 1008: Apply ArgoCD Application CR for the sigma1/infra Helm chart and validate full stack

## Objective
Create the ArgoCD Application custom resource pointing to the sigma1/infra Helm chart in the GitOps repo, sync it, and run the full validation suite confirming all pods, secrets, schemas, and the ConfigMap are healthy.

## Steps
Create sigma1/infra/argocd-application.yaml (applied outside the chart itself, or in an argocd/ directory). Spec: apiVersion: argoproj.io/v1alpha1, kind: Application, metadata.name: sigma1-infra, metadata.namespace: argocd. spec.source.repoURL: <GitOps repo URL>, spec.source.path: sigma1/infra, spec.source.targetRevision: HEAD. spec.destination.server: https://kubernetes.default.svc, spec.destination.namespace: sigma1. spec.syncPolicy.automated: {prune: true, selfHeal: true}. Apply: kubectl apply -f sigma1/infra/argocd-application.yaml -n argocd. Wait for sync: argocd app wait sigma1-infra --health --timeout 300. Run validation checklist: (a) kubectl get pods -n databases — sigma1-postgres-1 Running, sigma1-valkey-0 Running. (b) kubectl exec job/schema-init -n databases — confirm completed. (c) kubectl get secret -n sigma1 — 9 secrets present. (d) kubectl get pod -n signal — signal-cli Running. (e) kubectl get configmap sigma1-infra-endpoints -n sigma1 — 5 keys present. (f) ArgoCD UI or CLI shows Synced + Healthy.

## Validation
argocd app get sigma1-infra shows Health Status: Healthy and Sync Status: Synced. All 7 test_strategy checks from the parent task pass sequentially. CI pipeline step running kubectl get pods --all-namespaces | grep -E 'sigma1|databases|signal' shows no pods in CrashLoopBackOff or Error state.