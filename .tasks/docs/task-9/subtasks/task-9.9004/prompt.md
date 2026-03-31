Implement subtask 9004: Set up Cloudflare Tunnel ingress for application endpoints

## Objective
Deploy and configure a Cloudflare Tunnel (cloudflared) in the Kubernetes cluster to expose Morgan and web endpoints securely without opening inbound ports. Configure TLS enforcement on all routes.

## Steps
1. Create a Cloudflare Tunnel via the Cloudflare dashboard or API and obtain the tunnel credentials JSON.
2. Store the tunnel credentials as a Kubernetes Secret (`cloudflared-credentials`).
3. Create a `cloudflared` Deployment (2 replicas for HA) in the application namespace using the official `cloudflare/cloudflared` image.
4. Mount the credentials secret and a ConfigMap containing the tunnel config (`config.yaml`).
5. Define ingress rules in the tunnel config mapping hostnames to internal Kubernetes services:
   - `morgan.<domain>` → `http://morgan-service:PORT`
   - `app.<domain>` → `http://web-service:PORT`
   - Catch-all rule returning 404.
6. Configure `originRequest.noTLSVerify: false` to enforce TLS between cloudflared and origin if services use TLS internally, or `true` if traffic is cluster-internal HTTP.
7. Register DNS CNAME records in Cloudflare pointing to the tunnel UUID.
8. Apply resource limits and liveness probes to the cloudflared pods.

## Validation
Verify cloudflared pods are running and connected to the Cloudflare edge (check logs for 'Connection registered'). Confirm `https://morgan.<domain>` and `https://app.<domain>` resolve and return expected responses. Verify no NodePort or LoadBalancer services are exposed externally. Test that killing one cloudflared replica does not cause downtime.