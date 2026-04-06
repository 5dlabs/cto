Implement subtask 9005: Set up Cloudflare Tunnel ingress for Morgan and web frontend

## Objective
Deploy and configure a Cloudflare Tunnel (cloudflared) in the cluster to expose Morgan agent and web frontend services to the internet without opening inbound ports.

## Steps
1. Create a Cloudflare Tunnel via the Cloudflare dashboard or API and obtain the tunnel credentials JSON. 2. Store the tunnel credentials as a Kubernetes Secret. 3. Deploy `cloudflared` as a Deployment (2 replicas for HA) with the tunnel credentials mounted. 4. Create the cloudflared config.yaml mapping: `morgan.{domain}` → `http://morgan-service:PORT`, `www.{domain}` → `http://web-frontend-service:PORT`. 5. Configure DNS CNAME records in Cloudflare to point subdomains to the tunnel UUID. 6. Apply the Deployment and verify tunnel status shows 'healthy' in Cloudflare dashboard. 7. Test end-to-end access from external browser to both Morgan and web frontend.

## Validation
Verify `cloudflared` pods are running and connected (check logs for 'Connection registered'). Access morgan.{domain} and www.{domain} from an external network and confirm correct responses. Verify Cloudflare Tunnel dashboard shows healthy connectors. Kill one cloudflared pod and confirm the other maintains connectivity.