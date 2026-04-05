Implement subtask 1007: Deploy Cloudflare Tunnel for Morgan agent ingress

## Objective
Deploy a Cloudflare Tunnel (cloudflared) in the cluster to provide secure external ingress for the Morgan agent without exposing a public IP.

## Steps
1. Create a Cloudflare Tunnel via the Cloudflare dashboard or API, and obtain the tunnel token. 2. Store the tunnel credentials/token as a Kubernetes Secret 'sigma1-cloudflare-tunnel' in the 'sigma1' namespace. 3. Create a Deployment for cloudflared in the 'sigma1' namespace, mounting the tunnel credentials. 4. Configure the cloudflared config to route the desired hostnames to internal services (e.g., morgan.sigma1.svc.cluster.local). 5. Expose the tunnel as a ClusterIP Service if needed for health checks. 6. Verify DNS records in Cloudflare point to the tunnel. 7. Record the public ingress URL for the aggregated ConfigMap.

## Validation
cloudflared pod is Running; the configured public hostname resolves and returns a response from the target internal service; Cloudflare dashboard shows the tunnel as healthy.