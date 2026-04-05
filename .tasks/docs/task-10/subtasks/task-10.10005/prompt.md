Implement subtask 10005: Network policies: allow sigma1 services to sigma1-db namespace (PostgreSQL and Valkey)

## Objective
Create NetworkPolicy allowing sigma1 service pods to reach PostgreSQL on port 5432 and Valkey on port 6379 in the sigma1-db namespace.

## Steps
Step-by-step:
1. Create `netpol-allow-db.yaml` with two NetworkPolicy resources:
   a. **PostgreSQL egress** from sigma1:
      - `podSelector: {}` (all sigma1 pods)
      - `policyTypes: [Egress]`
      - `egress[0].to[0].namespaceSelector.matchLabels: {name: sigma1-db}`, `egress[0].ports: [{protocol: TCP, port: 5432}]`
   b. **Valkey egress** from sigma1:
      - Same structure but port 6379.
2. Create corresponding **ingress** policies in the `sigma1-db` namespace:
   a. Allow ingress from `sigma1` namespace pods on port 5432 for PostgreSQL pods.
   b. Allow ingress from `sigma1` namespace pods on port 6379 for Valkey pods.
3. Ensure namespaces have labels: `sigma1-db` namespace needs `name: sigma1-db`, `sigma1` namespace needs `name: sigma1`.
4. Also allow DNS egress (port 53 TCP/UDP to kube-system) so pods can resolve service names.

## Validation
From a test pod in sigma1 namespace, run `nc -zv <postgres-service>.sigma1-db.svc.cluster.local 5432` and verify connection succeeds. Run `nc -zv <valkey-service>.sigma1-db.svc.cluster.local 6379` and verify success. Attempt connection on port 3306 (MySQL) and verify it is denied.