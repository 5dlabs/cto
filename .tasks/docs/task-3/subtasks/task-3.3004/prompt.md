Implement subtask 3004: Implement namespace NetworkPolicies for default-deny and allowed traffic

## Objective
Create five NetworkPolicy resources: default deny all, allow app→PostgreSQL, allow app→Redis, allow ingress controller→app, and allow DNS egress.

## Steps
1. Create `infra/notifycore/templates/network-policies/` directory with separate files for clarity.
2. `default-deny.yaml`: NetworkPolicy with podSelector: {} (all pods), policyTypes: [Ingress, Egress], no ingress/egress rules (denies all by default).
3. `allow-app-to-pg.yaml`: NetworkPolicy selecting notifycore app pods (label: app=notifycore), allowing egress to pods matching cnpg.io/cluster=notifycore-pg on port 5432/TCP.
4. `allow-app-to-redis.yaml`: NetworkPolicy selecting notifycore app pods, allowing egress to pods matching app.kubernetes.io/name=redis on port 6379/TCP.
5. `allow-ingress-to-app.yaml`: NetworkPolicy selecting notifycore app pods, allowing ingress from namespaceSelector matching the ingress controller namespace (e.g., label: kubernetes.io/metadata.name=ingress-nginx) on port 8080/TCP.
6. `allow-dns-egress.yaml`: NetworkPolicy selecting all pods in namespace, allowing egress to kube-dns namespace on port 53 UDP and TCP.
7. Conditionally render network policies when `networkPolicies.enabled: true` in values.
8. `values-prod.yaml`: networkPolicies.enabled: true, ingressNamespace: ingress-nginx.
9. `values-dev.yaml`: networkPolicies.enabled: false.

## Validation
`helm template` with values-prod.yaml renders exactly 5 NetworkPolicy resources. Default deny policy has empty ingress and egress rules with both policyTypes. App-to-PG policy allows egress on port 5432. App-to-Redis allows egress on port 6379. Ingress-to-app allows ingress on 8080 from specific namespace. DNS policy allows egress on port 53. `values-dev.yaml` renders no NetworkPolicy resources.