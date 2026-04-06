Implement subtask 9003: Set up Cloudflare Tunnel ingress for Morgan agent and internal services

## Objective
Deploy and configure a Cloudflare Tunnel (cloudflared) within the cluster to securely expose the Morgan agent endpoint and any other internal services that need external reachability without a public IP or LoadBalancer.

## Steps
1) Create a Cloudflare Tunnel via the Cloudflare dashboard or `cloudflared tunnel create`. 2) Store the tunnel credentials JSON as a Kubernetes Secret. 3) Create a `cloudflared` Deployment (2 replicas for HA) in the cluster namespace with the tunnel credentials mounted. 4) Write the `config.yml` for cloudflared mapping public hostnames to internal Kubernetes service addresses (e.g., `morgan.example.com -> http://morgan-service.default.svc.cluster.local:PORT`). 5) Store the config as a ConfigMap and mount it into the cloudflared pods. 6) Create the corresponding CNAME DNS record in Cloudflare pointing to the tunnel UUID. 7) Verify the tunnel is healthy in the Cloudflare dashboard. 8) Add readiness probes to the cloudflared Deployment.

## Validation
Verify cloudflared pods are running and healthy via `kubectl get pods`. Confirm the Cloudflare Tunnel shows 'Healthy' status in the Cloudflare dashboard. Curl the Morgan agent's public hostname and verify a valid response is returned from the internal service. Test tunnel failover by deleting one cloudflared pod and confirming the other handles traffic without interruption.