## Decision Points

- Should sigma1-db be a new dedicated namespace or should the existing cluster-wide `databases` namespace be reused? This affects NetworkPolicy scope and RBAC boundaries.
- Signal-CLI sidecar template: should this be a Kustomize component (more composable, requires Kustomize adoption) or a ConfigMap-based template (simpler, but harder to inject as a container spec)? The choice affects how downstream services consume the sidecar definition.
- For sigma1-service-api-keys (inter-service auth per D7): how many service pairs need keys, and should keys be symmetric (shared secret) or asymmetric? This determines secret structure and rotation complexity.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm