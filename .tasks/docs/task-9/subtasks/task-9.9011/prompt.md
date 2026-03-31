Implement subtask 9011: Create Helm values-production.yaml overlay for all production configurations

## Objective
Extend the hermes-infra Helm chart with a values-production.yaml overlay that drives all production-specific configurations: replica counts, resource limits, TLS, ingress, network policies, HPA, and PDBs.

## Steps
1. Create `charts/hermes-infra/values-production.yaml` with all production-specific overrides.
2. Structure values by component: `postgresql.replicas: 3`, `redis.sentinel.enabled: true`, `nats.cluster.size: 3`, `minio.versioning: true`, etc.
3. Include TLS configuration: `tls.enabled: true`, `tls.issuerRef`, `tls.secretName`.
4. Include ingress configuration: `ingress.enabled: true`, `ingress.hosts`, `ingress.tls`, `ingress.annotations` (rate limiting, CORS).
5. Include network policy toggles: `networkPolicies.enabled: true`.
6. Include HPA and PDB configurations: `autoscaling.backend.enabled: true`, `autoscaling.backend.minReplicas: 2`, etc.
7. Update Helm templates to conditionally render production resources based on values.
8. Verify the chart renders correctly: `helm template hermes-infra charts/hermes-infra -f charts/hermes-infra/values-production.yaml` produces all expected resources.
9. Document the deployment command: `helm upgrade --install hermes-infra charts/hermes-infra -n hermes-production -f charts/hermes-infra/values-production.yaml`.

## Validation
Verify `helm template` with values-production.yaml renders all expected resources: 3-replica CNPG Cluster, Redis Sentinel CR, 3-node NATS CR, Ingress with TLS, NetworkPolicies, HPAs, and PDBs. Verify `helm lint charts/hermes-infra -f charts/hermes-infra/values-production.yaml` passes with no errors. Verify a dry-run install succeeds: `helm install --dry-run hermes-infra charts/hermes-infra -n hermes-production -f charts/hermes-infra/values-production.yaml`.