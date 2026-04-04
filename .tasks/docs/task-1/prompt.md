Implement task 1: Provision Dev Infrastructure for Sigma-1 E2E Pipeline (Bolt - Kubernetes/Helm)

## Goal
Bootstrap the sigma-1-dev namespace with all required infrastructure resources: namespace creation, Kubernetes secrets for Linear API key, Discord webhook URL, NOUS_API_KEY, and GitHub tokens, plus a sigma-1-infra-endpoints ConfigMap aggregating connection strings for all in-cluster services (discord-bridge-http, linear-bridge, openclaw-nats, cloudflare-operator). This task provides the foundational infrastructure that all subsequent tasks depend on.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
1. Create the `sigma-1-dev` namespace with appropriate labels (`project: sigma-1`, `env: dev`).
2. Create Kubernetes Secret `sigma-1-secrets` containing keys: `LINEAR_API_KEY`, `DISCORD_WEBHOOK_URL`, `NOUS_API_KEY`, `GITHUB_TOKEN`. Values should reference external-secrets operator CRs if available, otherwise placeholder sealed-secrets for dev.
3. Create ConfigMap `sigma-1-infra-endpoints` with the following keys:
   - `DISCORD_BRIDGE_URL`: internal cluster URL for `bots/discord-bridge-http` service
   - `LINEAR_BRIDGE_URL`: internal cluster URL for `bots/linear-bridge` service
   - `NATS_URL`: `openclaw-nats.openclaw.svc.cluster.local` (reference only; may not be used per D2 resolution)
   - `CLOUDFLARE_OPERATOR_NS`: `cloudflare-operator-system`
4. Create a ServiceAccount `sigma-1-pm-server` with minimal RBAC (get/list on configmaps and secrets in `sigma-1-dev` namespace only).
5. Validate that existing in-cluster services are reachable from the namespace: `bots/discord-bridge-http`, `bots/linear-bridge`, `cloudflare-operator-system` webhook service.
6. Create a Helm values file `values-dev.yaml` capturing all namespace-scoped resource names for downstream consumption.
7. Do NOT provision new NATS instances — NATS is already deployed at `openclaw-nats.openclaw.svc.cluster.local`. Do NOT deploy any ingress controller — Cloudflare operator handles ingress per D8.

## Acceptance Criteria
1. `kubectl get namespace sigma-1-dev` returns Active status. 2. `kubectl get secret sigma-1-secrets -n sigma-1-dev` exists and contains exactly 4 keys (LINEAR_API_KEY, DISCORD_WEBHOOK_URL, NOUS_API_KEY, GITHUB_TOKEN). 3. `kubectl get configmap sigma-1-infra-endpoints -n sigma-1-dev -o json` contains all 4 endpoint keys with non-empty values. 4. `kubectl get serviceaccount sigma-1-pm-server -n sigma-1-dev` exists. 5. A connectivity test pod in `sigma-1-dev` can resolve DNS for `discord-bridge-http`, `linear-bridge`, and `openclaw-nats.openclaw.svc.cluster.local`.

## Subtasks
- Create sigma-1-dev namespace with labels: Create the sigma-1-dev Kubernetes namespace with project and environment labels that all subsequent resources will be deployed into.
- Create sigma-1-secrets Kubernetes Secret with 4 keys: Create the Kubernetes Secret `sigma-1-secrets` in the sigma-1-dev namespace containing LINEAR_API_KEY, DISCORD_WEBHOOK_URL, NOUS_API_KEY, and GITHUB_TOKEN. Use external-secrets CRs if available, otherwise sealed-secrets placeholders for dev.
- Create sigma-1-infra-endpoints ConfigMap: Create the ConfigMap `sigma-1-infra-endpoints` in sigma-1-dev namespace with all 4 endpoint keys pointing to existing in-cluster services.
- Create ServiceAccount sigma-1-pm-server with RBAC Role and RoleBinding: Create a ServiceAccount, Role, and RoleBinding in sigma-1-dev namespace granting minimal get/list permissions on configmaps and secrets.
- Generate Helm values-dev.yaml capturing all resource names: Create a Helm values file that aggregates all namespace-scoped resource names (namespace, secret, configmap, service account) for downstream chart consumption.
- Validate cross-namespace connectivity from sigma-1-dev to existing services: Deploy a temporary test pod in sigma-1-dev namespace to verify DNS resolution and HTTP connectivity to discord-bridge-http, linear-bridge, and openclaw-nats services.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.