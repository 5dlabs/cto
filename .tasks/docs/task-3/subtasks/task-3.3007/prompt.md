Implement subtask 3007: Create values-prod.yaml consolidating all production settings

## Objective
Create the consolidated values-prod.yaml file integrating all HA, security, and scaling settings while ensuring values-dev.yaml remains unchanged.

## Steps
1. Create/finalize `infra/notifycore/values-prod.yaml` with all production settings:
   - postgres.instances: 3, postgres.minSyncReplicas: 1, postgres.backup: (configured)
   - redis.architecture: replication, redis.sentinel.enabled: true, redis.replica.replicaCount: 2
   - ingress.enabled: true, domain: example.com (placeholder)
   - networkPolicies.enabled: true, networkPolicies.ingressNamespace: ingress-nginx
   - hpa.enabled: true, hpa.minReplicas: 2, hpa.maxReplicas: 10, hpa.targetCPU: 70
   - resourceQuota.enabled: true
   - secretManagement.provider: none (placeholder)
2. Verify `values-dev.yaml` is unchanged from task 1 output (single-replica, no HA, no ingress, no network policies, no HPA, no resource quota).
3. Run `helm template infra/notifycore -f infra/notifycore/values-prod.yaml` and verify all expected resources are rendered.
4. Run `helm template infra/notifycore -f infra/notifycore/values-dev.yaml` and verify only dev resources are rendered.
5. Run `kubectl apply --dry-run=server` against a test cluster API for both value sets.

## Validation
`helm template` with values-prod.yaml renders all expected resources: 3-replica PG cluster, Redis sentinel, Ingress with TLS, 5 NetworkPolicies, HPA, ResourceQuota, ServiceAccount, PDBs. `helm template` with values-dev.yaml renders only basic resources (1-replica PG, standalone Redis, ConfigMap, no ingress/netpol/HPA/quota). `kubectl apply --dry-run=server` succeeds for both value sets.