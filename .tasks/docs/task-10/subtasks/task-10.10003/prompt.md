Implement subtask 10003: Define least-privilege RBAC Roles for PM server and frontend

## Objective
Create namespace-scoped Role resources implementing least-privilege access: PM server can read specific ConfigMaps and Secrets; frontend can only read ConfigMaps.

## Steps
1. Create `infra/rbac/roles.yaml`.
2. Define Role `pm-server-role` in namespace `sigma1-prod` with rules:
   - apiGroups: [""], resources: ["configmaps"], verbs: ["get", "list"], resourceNames: ["sigma1-infra-endpoints"].
   - apiGroups: [""], resources: ["secrets"], verbs: ["get"], resourceNames: ["linear-api-token", "github-pat", "discord-webhook-url", "nous-api-key"].
3. Define Role `frontend-role` in namespace `sigma1-prod` with rules:
   - apiGroups: [""], resources: ["configmaps"], verbs: ["get", "list"].
4. Apply: `kubectl apply -f infra/rbac/roles.yaml`.
5. Verify: `kubectl describe role pm-server-role -n sigma1-prod` and `kubectl describe role frontend-role -n sigma1-prod`.

## Validation
Run `kubectl describe role pm-server-role -n sigma1-prod` and confirm it grants get/list on configmaps (resourceNames: sigma1-infra-endpoints) and get on the 4 named secrets only. Run `kubectl describe role frontend-role -n sigma1-prod` and confirm it grants get/list on configmaps only with no secret access.