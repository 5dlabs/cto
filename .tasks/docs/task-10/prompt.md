Implement task 10: Production Hardening: HA, Ingress, Security, and RBAC (Bolt - Kubernetes/Helm)

## Goal
Harden the Sigma-1 dev infrastructure for production-readiness: scale CloudNative-PG and Redis to HA configurations, configure ingress with TLS termination, tighten Cilium network policies, enforce RBAC on all service accounts, enable secret rotation via external-secrets, and add audit logging. This task depends on all implementation and validation tasks completing successfully.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: medium
- Dependencies: 2, 3, 4, 5, 6, 7, 8, 9

## Implementation Plan
1. Scale CloudNative-PG cluster `sigma-1-pg` to 3 replicas (1 primary, 2 read replicas) with automated failover. Verify HA by checking `status.instances` and `status.readyInstances` match.
2. Scale Redis to a Sentinel or replicated configuration (3 nodes) if the operator supports it, or deploy a StatefulSet with Redis Sentinel. Update `sigma-1-infra-endpoints` ConfigMap with the Sentinel-aware connection string.
3. Configure ingress (Nginx Ingress Controller or existing cluster ingress) for the PM server with TLS termination using cert-manager or a pre-provisioned certificate. Set up the ingress resource with appropriate annotations for rate limiting and request size limits.
4. Tighten Cilium NetworkPolicies: review and restrict all policies to exact port numbers (not just pod selectors). Add explicit deny-all default policy for the namespace, then allowlist only required paths. Block all egress except to declared external API endpoints (Linear, GitHub, Hermes) and in-cluster services.
5. Enforce RBAC: create dedicated ClusterRoles/Roles for each service account. PM server SA: read ConfigMaps/Secrets in namespace. No service account should have cluster-admin or wildcard permissions. Audit existing bindings and remove any overly permissive ones.
6. Enable secret rotation: configure external-secrets ExternalSecret CRDs with `refreshInterval: 1h`. Add a rotation annotation or policy where the backing secret store supports it. Verify rotated secrets are picked up by pods without restart (or configure rolling restart triggers).
7. Add audit logging: enable Kubernetes audit logging for the namespace (if cluster-level audit policy allows namespace scoping) or add structured logging sidecar for secret access events.
8. Document all hardening changes in a `docs/production-hardening.md` file with the rationale for each change.

## Acceptance Criteria
1. CloudNative-PG: `status.readyInstances` == 3; simulate primary pod deletion and verify automatic failover within 30 seconds (new primary elected). 2. Redis: `redis-cli -h <sentinel> sentinel master sigma-1-redis` returns a valid master with 2 replicas. 3. Ingress: `curl -k https://<ingress-host>/api/pipeline/status` returns 200 with valid TLS certificate (verify with `openssl s_client`). 4. Network policies: attempt a connection from an unlisted pod to the PM server; assert connection is refused/timed out. Verify PM server can still reach Postgres, Redis, and bridge services. 5. RBAC: `kubectl auth can-i --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa create pods -n sigma-1-dev` returns `no`. 6. Secret rotation: modify a secret in the backing store; assert the Kubernetes Secret is updated within `refreshInterval` (verify by comparing resourceVersion before and after). 7. `docs/production-hardening.md` exists and documents at least 6 hardening measures.

## Subtasks
- Scale CloudNative-PG cluster to 3 replicas with automated failover: Update the CloudNative-PG Cluster CR `sigma-1-pg` to run 3 instances (1 primary, 2 read replicas) with automated failover enabled. Validate that all replicas reach ready state and streaming replication is active.
- Configure Redis HA with Sentinel and update ConfigMap connection string: Scale Redis to a Sentinel or replicated configuration with 3 nodes. Update the `sigma-1-infra-endpoints` ConfigMap with the Sentinel-aware connection string so consumers use the HA endpoint.
- Configure Ingress with TLS termination via cert-manager for PM server: Create an Ingress resource for the PM server with TLS termination using cert-manager. Add annotations for rate limiting and request size limits.
- Harden Cilium NetworkPolicies with deny-all default and exact port allowlisting: Create an explicit deny-all default CiliumNetworkPolicy for the sigma-1-dev namespace, then author allowlist policies with exact port numbers for all required ingress and egress paths.
- Audit and tighten RBAC with dedicated Roles for all service accounts: Audit all existing RoleBindings and ClusterRoleBindings in sigma-1-dev, remove overly permissive bindings, and create dedicated least-privilege Roles/RoleBindings for each service account.
- Configure secret rotation via external-secrets with refreshInterval and rolling restart triggers: Update all ExternalSecret CRDs to include `refreshInterval: 1h` for automatic rotation. Configure pod rolling restart triggers so rotated secrets are picked up without manual intervention.
- Add audit logging for namespace security events: Enable audit logging for the sigma-1-dev namespace to capture secret access events and security-relevant API calls, either via cluster audit policy or a structured logging sidecar.
- Document all production hardening changes in docs/production-hardening.md: Create a comprehensive `docs/production-hardening.md` document covering all hardening measures applied, including rationale, configuration details, and verification steps for each.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.