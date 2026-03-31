Implement subtask 1008: Package Helm chart with values overlays for dev and staging

## Objective
Structure and finalize the charts/hermes-infra Helm chart with values-dev.yaml and values-staging.yaml overlays, ensuring a single helm upgrade --install per environment deploys everything.

## Steps
1. Ensure `charts/hermes-infra/Chart.yaml` has correct metadata (name, version, appVersion, description).
2. Create `values.yaml` with sensible defaults shared across environments.
3. Create `values-dev.yaml` override: single replicas, smaller resource requests, 90-day MinIO retention, `ENVIRONMENT: dev`, automated ArgoCD sync.
4. Create `values-staging.yaml` override: single replicas (for now), larger resource requests, 365-day MinIO retention, `ENVIRONMENT: staging`, manual ArgoCD sync.
5. Add `templates/NOTES.txt` with post-install instructions.
6. Validate with `helm template charts/hermes-infra -f charts/hermes-infra/values-dev.yaml` and verify all templates render correctly.
7. Test with `helm upgrade --install hermes-infra charts/hermes-infra -n hermes-dev -f charts/hermes-infra/values-dev.yaml` to confirm a single command provisions everything.
8. Add a `README.md` in `charts/hermes-infra/` documenting the chart, values, and usage.

## Validation
`helm lint charts/hermes-infra` passes. `helm template` with both values files renders all expected resources without errors. `helm upgrade --install` on a clean namespace provisions all infrastructure components and the test pod connectivity check passes.