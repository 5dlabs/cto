<identity>
You are bolt working on subtask 10010 of task 10.
</identity>

<context>
<scope>
Deploy a cloudflared Deployment (1 replica) in the sigma1 namespace. Configure the tunnel to expose morgan-chat-svc:3000 → chat.sigma1.com and signal webhook → signals.sigma1.com. Store the tunnel credentials as a Kubernetes Secret. Enforce HTTPS-only at the Cloudflare dashboard. All services use ClusterIP only (no NodePort or LoadBalancer).
</scope>
</context>

<implementation_plan>
Create a Cloudflare Tunnel via `cloudflared tunnel create sigma1-tunnel`. Store the credentials JSON as `kubectl create secret generic cloudflare-tunnel-creds --from-file=credentials.json -n sigma1`. Create helm/sigma1/templates/cloudflared-deployment.yaml: `image: cloudflare/cloudflared:latest, args: ['tunnel', '--config', '/etc/cloudflared/config.yaml', 'run']`. Mount ConfigMap with config.yaml: `tunnel: <TUNNEL_ID>, credentials-file: /etc/cloudflared/creds/credentials.json, ingress: [{ hostname: chat.sigma1.com, service: http://morgan-svc:3000 }, { hostname: signals.sigma1.com, service: http://morgan-svc:3000/webhook/signal }, { service: http_status:404 }]`. Confirm all services have `type: ClusterIP`. Enable Cloudflare dashboard HTTPS redirect rule for both hostnames.
</implementation_plan>

<validation>
`curl https://chat.sigma1.com/health` returns HTTP 200 with Morgan health response. `curl https://signals.sigma1.com` returns expected response (not 404 or SSL error). `kubectl get svc -n sigma1` shows no LoadBalancer or NodePort types — all ClusterIP. TLS certificate shown by `openssl s_client -connect chat.sigma1.com:443` is valid Cloudflare-issued cert.
</validation>