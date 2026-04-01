## Provision Dev Infrastructure for Sigma-1 E2E Pipeline (Bolt - Kubernetes/Helm)

### Objective
Set up the development infrastructure required for the Sigma-1 agent delegation E2E pipeline. This includes creating a dedicated namespace, provisioning secrets for Linear API, Discord webhook, GitHub PAT, and NOUS_API_KEY, and publishing a sigma1-infra-endpoints ConfigMap aggregating all service connection strings so downstream tasks never re-provision infra.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
1. Create Kubernetes namespace `sigma1-dev`.
2. Create sealed secrets:
   - `linear-api-token` — Linear API key for issue creation and delegate resolution.
   - `discord-webhook-url` — Discord channel webhook for notifications.
   - `github-pat` — GitHub personal access token with repo scope for 5dlabs/sigma-1.
   - `nous-api-key` — Hermes/Nous research API key.
3. Deploy a ConfigMap `sigma1-infra-endpoints` with keys:
   - `PM_SERVER_URL` — internal service URL for the PM server.
   - `LINEAR_API_BASE` — https://api.linear.app/graphql.
   - `DISCORD_WEBHOOK_URL` — referenced from secret.
   - `GITHUB_API_BASE` — https://api.github.com.
   - `NOUS_API_BASE` — Hermes research endpoint.
4. Create a Helm values file `values-sigma1-dev.yaml` with single-replica deployments for the PM server and any auxiliary services.
5. Apply network policies allowing egress to Linear, Discord, GitHub, and Nous APIs.
6. Validate all secrets are mounted and ConfigMap is readable from a test pod.

### Subtasks
- [ ] Create sigma1-dev Kubernetes namespace: Create the dedicated `sigma1-dev` namespace that will host all Sigma-1 pipeline resources including secrets, ConfigMaps, and workloads.
- [ ] Provision sealed secrets for Linear API, Discord webhook, GitHub PAT, and NOUS_API_KEY: Create four SealedSecret resources in the sigma1-dev namespace for `linear-api-token`, `discord-webhook-url`, `github-pat`, and `nous-api-key`, ensuring each secret holds its respective API credential.
- [ ] Create sigma1-infra-endpoints ConfigMap: Deploy the `sigma1-infra-endpoints` ConfigMap in sigma1-dev containing all five service connection strings (PM_SERVER_URL, LINEAR_API_BASE, DISCORD_WEBHOOK_URL, GITHUB_API_BASE, NOUS_API_BASE) that downstream tasks consume via envFrom.
- [ ] Author Helm values file for single-replica dev deployments: Create `values-sigma1-dev.yaml` Helm values file configuring single-replica deployments for the PM server and any auxiliary services, referencing the sigma1-infra-endpoints ConfigMap and provisioned secrets.
- [ ] Apply network policies for egress to Linear, Discord, GitHub, and Nous APIs: Create and apply Kubernetes NetworkPolicy resources in sigma1-dev allowing egress traffic to Linear API, Discord webhooks, GitHub API, and Nous/Hermes API endpoints while denying all other egress by default.
- [ ] Validate infrastructure with a test pod: Deploy a temporary curl/test pod in sigma1-dev to validate that all secrets are mountable, the ConfigMap is readable, DNS resolution works, and network policies allow expected egress while blocking unexpected traffic.