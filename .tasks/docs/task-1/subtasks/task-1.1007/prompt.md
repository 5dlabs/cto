Implement subtask 1007: Create ArgoCD Application CRs for backend and frontend in both environments

## Objective
Create ArgoCD Application custom resources for hermes-backend and hermes-frontend in both dev and staging namespaces with appropriate sync policies.

## Steps
1. Create Helm templates for ArgoCD Application CRs in `charts/hermes-infra/templates/argocd-apps.yaml`.
2. Create four Application CRs:
   - `hermes-backend-dev` — targets `hermes-dev` namespace, automated sync policy
   - `hermes-backend-staging` — targets `hermes-staging` namespace, manual sync with auto-prune
   - `hermes-frontend-dev` — targets `hermes-dev` namespace, automated sync policy
   - `hermes-frontend-staging` — targets `hermes-staging` namespace, manual sync with auto-prune
3. Configure source repository, path, and target revision as Helm values.
4. For staging apps, add annotations for E2E test gating hook (placeholder annotation `hermes.openclaw.io/e2e-gate: enabled` for Task 7 integration).
5. Ensure Application CRs reference the correct project in ArgoCD (create an ArgoCD `AppProject` if needed to scope permissions).

## Validation
ArgoCD UI shows all four Application CRs (`hermes-backend-dev`, `hermes-backend-staging`, `hermes-frontend-dev`, `hermes-frontend-staging`). Dev apps have automated sync policy. Staging apps have manual sync with auto-prune. All apps show Synced/Healthy state (initially empty target).