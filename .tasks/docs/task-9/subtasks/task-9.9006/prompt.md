Implement subtask 9006: Configure TLS termination with cert-manager and HTTPS enforcement

## Objective
Set up TLS for all public Hermes endpoints using cert-manager with Let's Encrypt (or pre-provisioned TLS secrets), and enforce HTTPS redirect on all HTTP endpoints.

## Steps
1. If cert-manager is available: create a `ClusterIssuer` or `Issuer` resource for Let's Encrypt (staging first, then production).
2. Create a `Certificate` CR for the Hermes domain(s): `hermes.{domain}` and `hermes-api.{domain}` (or a wildcard if using subdomain routing).
3. If cert-manager is not available: create a TLS Secret manually from pre-provisioned certificate and key files.
4. Configure the ingress controller to enforce HTTPS redirect: `nginx.ingress.kubernetes.io/ssl-redirect: "true"` (for nginx-ingress) or equivalent annotation for the cluster's ingress controller.
5. Set minimum TLS version to 1.2: `nginx.ingress.kubernetes.io/ssl-protocols: "TLSv1.2 TLSv1.3"`.
6. Store TLS configuration references (secret name, issuer name) for use by the Ingress subtask.

## Validation
Verify the Certificate CR status shows `Ready=True`: `kubectl get certificate -n hermes-production`. Verify TLS secret is created: `kubectl get secret hermes-tls -n hermes-production`. Verify `curl -v https://hermes.{domain}` shows TLS 1.2+ handshake. Verify `curl http://hermes.{domain}` returns 301 redirect to HTTPS.