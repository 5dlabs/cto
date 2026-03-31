Implement subtask 9007: Create Ingress resources with routing rules, rate limiting, and CORS

## Objective
Create Kubernetes Ingress resources for production routing: frontend and API endpoints with rate limiting annotations and CORS headers for frontend-to-API communication.

## Steps
1. Create Ingress resource for the Next.js frontend: host `hermes.{domain}`, path `/` → frontend Service on port 3000.
2. Create Ingress resource (or path rule) for the Bun/Elysia backend: either `hermes-api.{domain}` (subdomain) or `hermes.{domain}/api/*` (path-based) → backend Service on port 3001.
3. Reference the TLS secret from subtask 9006 in `spec.tls`.
4. Add rate limiting annotations for nginx-ingress: `nginx.ingress.kubernetes.io/limit-rps: "50"`, `nginx.ingress.kubernetes.io/limit-connections: "20"`.
5. Add CORS annotations: `nginx.ingress.kubernetes.io/enable-cors: "true"`, `nginx.ingress.kubernetes.io/cors-allow-origin: "https://hermes.{domain}"`, `nginx.ingress.kubernetes.io/cors-allow-methods: "GET, POST, PUT, DELETE, OPTIONS"`.
6. Set appropriate proxy buffer sizes and timeouts for the API endpoints (deliberation responses may be large).

## Validation
Verify `curl https://hermes.{domain}/` returns Next.js HTML. Verify `curl https://hermes.{domain}/api/hermes/deliberations` (or `hermes-api.{domain}/hermes/deliberations`) returns JSON from the backend. Verify CORS headers are present: `curl -I -H 'Origin: https://hermes.{domain}' https://hermes-api.{domain}` shows `Access-Control-Allow-Origin`. Verify rate limiting: send 100 rapid requests and confirm some return 429.