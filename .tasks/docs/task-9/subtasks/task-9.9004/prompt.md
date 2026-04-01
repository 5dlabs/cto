Implement subtask 9004: Create Ingress resource for PM server API with TLS and rate limiting

## Objective
Define an Ingress manifest for `api.sigma1.5dlabs.io` with cert-manager TLS and rate-limiting annotations.

## Steps
1. Create `templates/pm-server-ingress.yaml`.
2. Set `apiVersion: networking.k8s.io/v1`, kind `Ingress`.
3. Add annotation `cert-manager.io/cluster-issuer: letsencrypt-prod` (or the resolved ClusterIssuer name).
4. Add rate-limiting annotations appropriate to the selected ingress controller (e.g., for nginx: `nginx.ingress.kubernetes.io/limit-rps: "100"`, `nginx.ingress.kubernetes.io/limit-connections: "50"`).
5. Define `spec.tls` with host `api.sigma1.5dlabs.io` and secretName `api-sigma1-tls`.
6. Define `spec.rules` with host `api.sigma1.5dlabs.io`, path `/` routing to the PM server Service on port 3000.
7. Parameterize the host, ClusterIssuer name, and rate-limit values via Helm values.
8. Validate with `helm template`.

## Validation
Rendered template contains the correct host, TLS secret, cert-manager annotation, and rate-limiting annotations. After deploy: `kubectl get ingress -n sigma1-prod` shows the API ingress with TLS configured and ADDRESS populated.