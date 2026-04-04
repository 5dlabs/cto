Implement subtask 10011: Security review of RBAC policies and NetworkPolicy configurations

## Objective
Review all RBAC Role/RoleBinding manifests and NetworkPolicy manifests for correctness, least-privilege compliance, and absence of over-permissive rules.

## Steps
1. Review `manifests/production/rbac-pm-server.yaml`:
   - Confirm it uses `Role` (not `ClusterRole`).
   - Confirm verbs are read-only (get, list, watch) — no create, update, delete, patch.
   - Confirm namespace is scoped to sigma-1-dev only.
2. Review `manifests/production/rbac-frontend.yaml`:
   - Confirm the frontend ServiceAccount cannot access secrets.
   - Confirm no ClusterRoleBinding exists.
3. Review `manifests/production/network-policies.yaml`:
   - Confirm default-deny egress policy exists.
   - Confirm PM server egress is limited to bridge services and required external APIs.
   - Confirm frontend egress is limited to PM server only.
   - Check for accidental ingress rules that might be too permissive.
4. Run `kubectl auth can-i --list --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server -n sigma-1-dev` and verify the output matches expected permissions.
5. Run same for the frontend ServiceAccount.
6. Document any findings or recommended changes.

## Validation
Review report is produced confirming: (a) no ClusterRole or ClusterRoleBinding references exist for either ServiceAccount, (b) PM server SA has only get/list/watch on configmaps and secrets, (c) frontend SA has only get/list on configmaps, (d) NetworkPolicies enforce default-deny with explicit allow rules only, (e) `kubectl auth can-i --list` output matches expected permissions for both SAs.