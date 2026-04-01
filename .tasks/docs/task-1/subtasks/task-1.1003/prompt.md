Implement subtask 1003: Create sigma1-infra-endpoints ConfigMap

## Objective
Deploy the `sigma1-infra-endpoints` ConfigMap in sigma1-dev containing all five service connection strings (PM_SERVER_URL, LINEAR_API_BASE, DISCORD_WEBHOOK_URL, GITHUB_API_BASE, NOUS_API_BASE) that downstream tasks consume via envFrom.

## Steps
1. Author `configmap-sigma1-infra-endpoints.yaml` with:
   - `PM_SERVER_URL`: internal K8s service URL for the PM server (e.g., `http://pm-server.sigma1-dev.svc.cluster.local:8080`).
   - `LINEAR_API_BASE`: `https://api.linear.app/graphql`.
   - `DISCORD_WEBHOOK_URL`: value referencing the convention that pods mount the `discord-webhook-url` secret separately; set to a placeholder note or the secret-reference pattern your stack uses.
   - `GITHUB_API_BASE`: `https://api.github.com`.
   - `NOUS_API_BASE`: Hermes research endpoint URL.
2. Apply: `kubectl apply -f configmap-sigma1-infra-endpoints.yaml -n sigma1-dev`.
3. Ensure the ConfigMap is consumable via `envFrom` by downstream Deployments.

## Validation
`kubectl get configmap sigma1-infra-endpoints -n sigma1-dev -o json | jq '.data'` contains exactly the five keys: PM_SERVER_URL, LINEAR_API_BASE, DISCORD_WEBHOOK_URL, GITHUB_API_BASE, NOUS_API_BASE, each with non-empty values.