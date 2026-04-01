Implement subtask 9005: Create Ingress resource for web frontend with TLS and CDN cache headers

## Objective
Define an Ingress manifest for `sigma1.5dlabs.io` with cert-manager TLS and CDN-friendly cache-control headers for static assets.

## Steps
1. Create `templates/frontend-ingress.yaml`.
2. Set `apiVersion: networking.k8s.io/v1`, kind `Ingress`.
3. Add annotation `cert-manager.io/cluster-issuer: letsencrypt-prod`.
4. Add cache-header annotations (e.g., for nginx: `nginx.ingress.kubernetes.io/configuration-snippet` with `add_header Cache-Control "public, max-age=3600";` scoped to static asset paths like `/_next/static`, `/static`).
5. Define `spec.tls` with host `sigma1.5dlabs.io` and secretName `sigma1-frontend-tls`.
6. Define `spec.rules` with host `sigma1.5dlabs.io`, path `/` routing to the frontend Service on port 3000.
7. Parameterize the host and ClusterIssuer via Helm values.
8. Validate with `helm template`.

## Validation
Rendered template contains correct host, TLS config, and cache-control annotation snippet. After deploy: `curl -I https://sigma1.5dlabs.io/_next/static/somefile.js` returns `Cache-Control: public, max-age=3600` header.