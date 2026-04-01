Implement task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Finalize production hardening by implementing least-privilege RBAC for all service accounts, automated secret rotation policies, and audit logging for security and compliance of the Sigma-1 pipeline infrastructure.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: medium
- Dependencies: 9

## Implementation Plan
1. Create dedicated ServiceAccounts for each deployment:
   - `sa-pm-server` — for PM server pods.
   - `sa-frontend` — for frontend pods.
2. Create RBAC Roles scoped to `sigma1-prod` namespace:
   - `pm-server-role`: get/list ConfigMaps (`sigma1-infra-endpoints`), get Secrets (only the 4 pipeline secrets).
   - `frontend-role`: get/list ConfigMaps only.
3. Create RoleBindings binding each ServiceAccount to its Role.
4. Ensure no pod uses the `default` ServiceAccount — update all Deployment specs.
5. Implement secret rotation using External Secrets Operator or a CronJob:
   a. `linear-api-token`: rotate every 90 days.
   b. `github-pat`: rotate every 90 days.
   c. `discord-webhook-url`: rotate every 180 days.
   d. `nous-api-key`: rotate every 90 days.
   e. On rotation, trigger a rolling restart of affected deployments.
6. Enable Kubernetes audit logging:
   a. Create an audit policy that logs all create/update/delete operations in `sigma1-prod`.
   b. Log authentication failures at the RequestResponse level.
   c. Route audit logs to a persistent volume or external log aggregator.
7. Add NetworkPolicy restricting inter-pod communication:
   - Frontend pods can only reach PM server on port 3000.
   - PM server can reach external APIs (Linear, GitHub, Discord, Nous) but not other namespaces.
8. Document all RBAC roles and rotation schedules in a `docs/security.md` file committed to the repo.

## Acceptance Criteria
1. RBAC verification: exec into a PM server pod and attempt to list pods — should receive 403 Forbidden. 2. RBAC verification: exec into a PM server pod and read `sigma1-infra-endpoints` ConfigMap — should succeed. 3. ServiceAccount audit: `kubectl get pods -n sigma1-prod -o jsonpath='{.items[*].spec.serviceAccountName}'` returns only `sa-pm-server` and `sa-frontend`, never `default`. 4. Secret rotation: manually trigger rotation CronJob; verify new secret value is mounted in pods after rolling restart (check pod restart timestamp). 5. Audit logging: perform a kubectl create in sigma1-prod; verify the action appears in audit log within 60 seconds. 6. NetworkPolicy: from a frontend pod, attempt to curl an external API directly — should be blocked; curl PM server on port 3000 — should succeed. 7. `docs/security.md` exists and contains RBAC role descriptions and rotation schedule table.

## Subtasks
- Create dedicated ServiceAccounts for PM server and frontend: Define and apply dedicated ServiceAccount resources for each workload in the sigma1-prod namespace, ensuring no pod uses the default ServiceAccount.
- Update all Deployment specs to use dedicated ServiceAccounts: Modify every Deployment in sigma1-prod to reference its dedicated ServiceAccount, removing any reliance on the default ServiceAccount.
- Define least-privilege RBAC Roles for PM server and frontend: Create namespace-scoped Role resources implementing least-privilege access: PM server can read specific ConfigMaps and Secrets; frontend can only read ConfigMaps.
- Create RoleBindings linking ServiceAccounts to their Roles: Bind each dedicated ServiceAccount to its corresponding least-privilege Role via RoleBinding resources.
- Implement automated secret rotation via CronJob with rolling restart triggers: Create CronJob resources that rotate each pipeline secret on its defined schedule and trigger rolling restarts of affected Deployments.
- Configure Kubernetes audit policy for sigma1-prod namespace: Create and apply a Kubernetes audit policy that logs all mutating operations in sigma1-prod and authentication failures at RequestResponse level, routing logs to persistent storage.
- Create NetworkPolicy restricting inter-pod and egress communication: Define and apply NetworkPolicy resources that restrict frontend pods to only reach PM server on port 3000, and restrict PM server egress to external APIs while blocking cross-namespace traffic.
- Create security documentation in docs/security.md: Document all RBAC roles, ServiceAccount assignments, secret rotation schedules, NetworkPolicy rules, and audit logging configuration for operational reference and compliance.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.