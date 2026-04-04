Implement subtask 1005: Generate Helm values-dev.yaml capturing all resource names

## Objective
Create a Helm values file that aggregates all namespace-scoped resource names (namespace, secret, configmap, service account) for downstream chart consumption.

## Steps
1. Create `values-dev.yaml` in the project's Helm chart directory.
2. Include the following keys:
   - `namespace: sigma-1-dev`
   - `secrets.name: sigma-1-secrets`
   - `configmap.endpoints: sigma-1-infra-endpoints`
   - `serviceAccount.name: sigma-1-pm-server`
   - `externalServices.discordBridge`, `externalServices.linearBridge`, `externalServices.nats`, `externalServices.cloudflareOperatorNs` with their respective URLs/values
3. Ensure the values file is valid YAML and can be consumed by `helm template` without errors.
4. Add a comment block at the top explaining this is the dev environment values file for the sigma-1 pipeline.

## Validation
`helm template . -f values-dev.yaml` runs without errors. The values file contains keys for namespace, secrets.name, configmap.endpoints, and serviceAccount.name with correct string values matching the created resources.