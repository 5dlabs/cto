Implement subtask 1012: Create NetworkPolicy allowing sigma1 to sigma1-db ingress

## Objective
Create a NetworkPolicy in the sigma1-db namespace that allows ingress traffic from pods in the sigma1 namespace to the PgBouncer pooler and Valkey services, while denying other ingress by default.

## Steps
1. Create a default-deny ingress NetworkPolicy in `sigma1-db` namespace (if not already present):
   - `spec.podSelector: {}` (all pods)
   - `spec.policyTypes: [Ingress]`
   - No ingress rules (deny all by default)
2. Create an allow NetworkPolicy `allow-sigma1-to-db` in `sigma1-db` namespace:
   - `spec.podSelector: {}` (applies to all pods in sigma1-db)
   - `spec.ingress[0].from[0].namespaceSelector.matchLabels: { app.kubernetes.io/part-of: sigma1 }`
   - `spec.ingress[0].ports`: allow TCP 5432 (PgBouncer) and TCP 6379 (Valkey)
3. Apply both NetworkPolicy YAMLs.
4. Verify from a pod in sigma1: connections to PgBouncer on 5432 and Valkey on 6379 succeed.
5. Optionally verify from a pod in a different namespace: connections are denied.

## Validation
From a temporary pod in sigma1 namespace: `nc -zv sigma1-postgres-pooler.sigma1-db.svc.cluster.local 5432` succeeds and `nc -zv sigma1-valkey.sigma1-db.svc.cluster.local 6379` succeeds. From a pod in `default` namespace: the same connections time out or are refused. `kubectl get networkpolicy -n sigma1-db` lists both policies.