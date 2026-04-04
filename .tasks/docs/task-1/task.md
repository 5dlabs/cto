## Provision Dev Infrastructure for Sigma-1 E2E Pipeline (Bolt - Kubernetes/Helm)

### Objective
Bootstrap the dev namespace and all in-cluster infrastructure required by the Sigma-1 E2E pipeline. This includes CloudNative-PG Postgres, Redis, external-secrets ExternalSecret CRDs for all four sensitive tokens (Linear API, GitHub API, NOUS_API_KEY, Discord webhook / service-to-service API key), Cilium network policies restricting pod-to-pod traffic to declared paths, and a sigma-1-infra-endpoints ConfigMap aggregating all connection strings. All subsequent tasks depend on this foundation.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
1. Create namespace `sigma-1-dev` with resource quotas and limit ranges appropriate for a validation run.
2. Deploy CloudNative-PG Cluster CR (`sigma-1-pg`, single replica, 1Gi storage) and wait for the `Ready` condition.
3. Deploy Redis CR or StatefulSet (`sigma-1-redis`, single replica, no persistence needed for validation).
4. Create ExternalSecret CRDs referencing the backing secret store for: `LINEAR_API_TOKEN`, `GITHUB_API_TOKEN`, `NOUS_API_KEY`, `DISCORD_WEBHOOK_URL`, `SERVICE_API_KEY` (shared service-to-service key per D5). Verify external-secrets operator syncs each to a Kubernetes Secret.
5. Verify connectivity to the external secret store by checking each ExternalSecret's `status.conditions` for `Ready=True`.
6. Create ConfigMap `sigma-1-infra-endpoints` with keys: `CNPG_SIGMA1_PG_URL` (Postgres connection string from CNPG secret), `REDIS_SIGMA1_URL` (Redis service URL), `DISCORD_BRIDGE_URL` (in-cluster `bots/discord-bridge-http` service URL), `LINEAR_BRIDGE_URL` (in-cluster `bots/linear-bridge` service URL).
7. Apply Cilium NetworkPolicy CRDs: allow PM server → Postgres, PM server → Redis, PM server → discord-bridge-http, PM server → linear-bridge, PM server → egress for Linear/GitHub/Hermes APIs. Deny all other intra-namespace traffic.
8. Create a ServiceAccount `sigma-1-pm-sa` with minimal RBAC (read ConfigMaps and Secrets in namespace only).
9. Validate: run a smoke Pod that mounts the ConfigMap via `envFrom`, connects to Postgres and Redis, and resolves bridge service DNS names.

### Subtasks
- [ ] Create sigma-1-dev namespace with resource quotas and limit ranges: Create the Kubernetes namespace `sigma-1-dev` and apply ResourceQuota and LimitRange objects scoped for a dev validation run, ensuring pods cannot exceed reasonable CPU/memory bounds.
- [ ] Deploy CloudNative-PG Postgres cluster CR and validate readiness: Deploy a single-replica CloudNative-PG Cluster custom resource named `sigma-1-pg` in the `sigma-1-dev` namespace with 1Gi storage, and wait until the cluster reaches a healthy state.
- [ ] Deploy Redis single-replica instance: Deploy a single-replica Redis instance named `sigma-1-redis` in the `sigma-1-dev` namespace with no persistence, and expose it via a ClusterIP Service.
- [ ] Create ExternalSecret CRDs for LINEAR_API_TOKEN and GITHUB_API_TOKEN: Create ExternalSecret CRDs that reference the cluster's backing secret store to sync `LINEAR_API_TOKEN` and `GITHUB_API_TOKEN` into Kubernetes Secrets in the `sigma-1-dev` namespace.
- [ ] Create ExternalSecret CRDs for NOUS_API_KEY, DISCORD_WEBHOOK_URL, and SERVICE_API_KEY: Create ExternalSecret CRDs that reference the cluster's backing secret store to sync `NOUS_API_KEY`, `DISCORD_WEBHOOK_URL`, and `SERVICE_API_KEY` into Kubernetes Secrets in the `sigma-1-dev` namespace.
- [ ] Create ServiceAccount sigma-1-pm-sa with minimal RBAC: Create a Kubernetes ServiceAccount `sigma-1-pm-sa` in the `sigma-1-dev` namespace with a Role and RoleBinding granting read-only access to ConfigMaps and Secrets within the namespace only.
- [ ] Create sigma-1-infra-endpoints ConfigMap with all connection strings: Create the ConfigMap `sigma-1-infra-endpoints` in `sigma-1-dev` aggregating the Postgres connection string from the CNPG-generated secret, the Redis service URL, and the in-cluster URLs for the discord-bridge-http and linear-bridge services.
- [ ] Apply Cilium NetworkPolicy CRDs for PM server traffic paths: Create and apply Cilium NetworkPolicy (CiliumNetworkPolicy) CRDs in the `sigma-1-dev` namespace that allow only declared traffic paths from the PM server to Postgres, Redis, discord-bridge-http, and linear-bridge, plus egress to external Linear/GitHub/Hermes APIs. Deny all other intra-namespace traffic.
- [ ] Deploy smoke test Pod to validate end-to-end infrastructure connectivity: Create and run a smoke test Pod in `sigma-1-dev` that mounts the `sigma-1-infra-endpoints` ConfigMap via envFrom, uses the `sigma-1-pm-sa` ServiceAccount, connects to Postgres (SELECT 1), pings Redis (PING→PONG), and resolves both bridge service DNS names. The Pod must exit 0 on success.