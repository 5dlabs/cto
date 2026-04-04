Implement subtask 2001: Configure ConfigMap and secret mounting for PM server deployment

## Objective
Update the PM server Kubernetes deployment manifest to mount the sigma-1-infra-endpoints ConfigMap and external-secrets-managed secrets via envFrom, making Linear API tokens and infra endpoints available as environment variables.

## Steps
1. Open the PM server deployment YAML (e.g., `k8s/pm-server/deployment.yaml`).
2. Add an `envFrom` entry referencing the `sigma-1-infra-endpoints` ConfigMap.
3. Add an `envFrom` entry referencing the ExternalSecret-managed secret (e.g., `sigma-1-secrets`) that contains the Linear API key.
4. Verify env vars are typed in a shared `env.ts` file using Bun's `process.env` or `Bun.env`, exporting typed constants like `LINEAR_API_KEY`, `LINEAR_TEAM_ID`.
5. Add validation at server startup that all required env vars are present; throw a clear error if any are missing.

## Validation
Deploy the updated manifest to a dev namespace. Exec into the pod and verify `echo $LINEAR_API_KEY` and `echo $LINEAR_TEAM_ID` return non-empty values. Verify server startup logs confirm all required env vars are present.