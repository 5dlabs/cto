Implement subtask 1004: Author Helm values file for single-replica dev deployments

## Objective
Create `values-sigma1-dev.yaml` Helm values file configuring single-replica deployments for the PM server and any auxiliary services, referencing the sigma1-infra-endpoints ConfigMap and provisioned secrets.

## Steps
1. Create `values-sigma1-dev.yaml` in the Helm chart directory.
2. Set `replicaCount: 1` for PM server and any auxiliary services.
3. Configure `envFrom` to include `configMapRef: sigma1-infra-endpoints`.
4. Configure secret volume mounts or `envFrom` secretRefs for `linear-api-token`, `discord-webhook-url`, `github-pat`, `nous-api-key`.
5. Set resource requests/limits appropriate for dev (e.g., 128Mi–256Mi memory, 100m–250m CPU).
6. Ensure image tags reference a valid dev image or placeholder.
7. Validate the values file renders correctly: `helm template sigma1 ./chart -f values-sigma1-dev.yaml --namespace sigma1-dev` produces valid manifests without errors.

## Validation
`helm template sigma1 ./chart -f values-sigma1-dev.yaml --namespace sigma1-dev` renders without errors. Rendered Deployment manifests show `replicaCount: 1`, correct envFrom referencing `sigma1-infra-endpoints`, and secretRef entries for all four secrets.