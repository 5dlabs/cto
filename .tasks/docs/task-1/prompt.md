Implement task 1: Provision Dev Infrastructure for Sigma-1 E2E Pipeline (Bolt - Kubernetes/Helm)

## Goal
Bootstrap the sigma-1 namespace with all required infrastructure resources: ExternalSecret CRDs for NOUS_API_KEY, Linear API token, Discord webhook URL, and GitHub token; a ConfigMap aggregating service endpoints; and validation that all secrets resolve to non-empty values before downstream tasks proceed.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
Step-by-step implementation:

1. Create namespace `sigma-1` with standard labels (`app.kubernetes.io/part-of: sigma-1`, `env: dev`).

2. Create ExternalSecret CRDs referencing the cluster's existing SecretStore:
   - `sigma-1-linear-token` → Linear API token
   - `sigma-1-discord-webhook` → Discord webhook URL
   - `sigma-1-nous-api-key` → NOUS_API_KEY (mark as optional — pipeline must not fail if absent)
   - `sigma-1-github-token` → GitHub PAT for 5dlabs/sigma-1 repo

3. Create a Kubernetes Job or init-container script (`secret-validation-job`) that:
   a. Waits up to 60s for ExternalSecret resources to sync
   b. Reads each resulting Secret and asserts the data field is non-empty
   c. For `sigma-1-nous-api-key`: log a warning if empty but do NOT fail (consistent with D8 graceful skip)
   d. For all other secrets: fail with a clear error message identifying which secret path is missing
   e. Exits 0 only when all required secrets are populated

4. Create ConfigMap `sigma-1-infra-endpoints` with keys:
   - `DISCORD_BRIDGE_URL` → `http://discord-bridge-http.bots.svc.cluster.local`
   - `LINEAR_BRIDGE_URL` → `http://linear-bridge.bots.svc.cluster.local`
   - `PM_SERVER_URL` → `http://cto-pm.cto.svc.cluster.local`
   - `HERMES_URL` → discovered Hermes endpoint or empty string
   - `NOUS_API_URL` → `https://api.nous.com` (or appropriate external endpoint)

5. Verify existing services are reachable from the namespace:
   - `bots/discord-bridge-http` responds to health check
   - `bots/linear-bridge` responds to health check
   - `cto/cto-pm` responds to health check

6. Label all resources with `sigma-1-pipeline: infra` for cleanup traceability.

7. Document in a README any backing store paths that need manual pre-configuration if not already present (Open Question #1).

## Acceptance Criteria
1. `kubectl get ns sigma-1` returns Active status with expected labels. 2. `kubectl get externalsecrets -n sigma-1` shows all 4 ExternalSecret resources with status 'SecretSynced'. 3. Secret validation job completes with exit code 0, and logs confirm non-empty values for linear-token, discord-webhook, and github-token. 4. `kubectl get configmap sigma-1-infra-endpoints -n sigma-1 -o json` contains all 5 expected keys with non-empty values for DISCORD_BRIDGE_URL, LINEAR_BRIDGE_URL, and PM_SERVER_URL. 5. Health check probes to discord-bridge-http, linear-bridge, and cto-pm return 200 from within the sigma-1 namespace.

## Subtasks
- Create sigma-1 namespace with standard labels: Create the Kubernetes namespace `sigma-1` with the required labels for project identification and environment tagging. This is the foundational resource all other subtasks depend on.
- Create ExternalSecret CRD for sigma-1-linear-token: Define and apply the ExternalSecret resource for the Linear API token, referencing the cluster's existing SecretStore and targeting the correct backing store path.
- Create ExternalSecret CRD for sigma-1-discord-webhook: Define and apply the ExternalSecret resource for the Discord webhook URL, referencing the cluster's existing SecretStore.
- Create ExternalSecret CRD for sigma-1-nous-api-key (optional): Define and apply the ExternalSecret resource for NOUS_API_KEY, marked as optional so that its absence does not block the pipeline.
- Create ExternalSecret CRD for sigma-1-github-token: Define and apply the ExternalSecret resource for the GitHub PAT used to access the 5dlabs/sigma-1 repository.
- Implement secret-validation-job with conditional logic for optional NOUS_API_KEY: Create a Kubernetes Job that waits for ExternalSecrets to sync, validates that all required secrets are non-empty, warns (but does not fail) if the optional NOUS_API_KEY is absent, and fails clearly if any required secret is missing.
- Create sigma-1-infra-endpoints ConfigMap with service endpoints: Create the ConfigMap `sigma-1-infra-endpoints` containing all service endpoint URLs that downstream tasks will consume via `envFrom`.
- Verify cross-namespace service health checks from sigma-1: Run health check probes from within the sigma-1 namespace to confirm that discord-bridge-http, linear-bridge, and cto-pm services in their respective namespaces are reachable and responding.
- Document backing store paths and manual prerequisites in README: Write a README documenting all backing store secret paths that must be pre-configured, the ExternalSecret-to-SecretStore mapping, the ConfigMap contract for downstream consumers, and any manual steps required before running the infrastructure provisioning.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.