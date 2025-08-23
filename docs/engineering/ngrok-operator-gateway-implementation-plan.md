## Ngrok Kubernetes Operator with Gateway API — Implementation Plan (public.5dlabs.ai)



### Goals


- **Install** the ngrok Kubernetes Operator with **Gateway API** support


- **Use** custom domain `public.5dlabs.ai`


- **Route** GitHub webhooks to Argo Events (verified, then forwarded)


- **Enable** publishing of additional public-facing apps via routes/hosts



### Prerequisites
- ngrok account with:


  - **Authtoken** and **API key** (dashboard links below)


- A Kubernetes cluster with `kubectl` and `helm` available


- DNS control for `5dlabs.ai` to create a CNAME for `public.5dlabs.ai`


- Argo Events installed (or planned) with a GitHub `EventSource` Service to receive webhooks

Reference docs (from local clone):
- Helm configuration: `docs/ngrok-docs/docs/k8s/installation/helm.mdx`
- Gateway API quickstart: `docs/ngrok-docs/docs/getting-started/kubernetes/gateway-api.mdx`
- Using Gateway API: `docs/ngrok-docs/docs/k8s/guides/using-gwapi.mdx`
- Custom domains: `docs/ngrok-docs/docs/k8s/guides/custom-domain.mdx`
- Webhook verification (Traffic Policy): `docs/ngrok-docs/traffic-policy/actions/verify-webhook/`

### Phase 1 — Install Operator with Gateway API
1) Add Helm repo and export credentials





```bash
helm repo add ngrok https://charts.ngrok.com
helm repo update

# From ngrok dashboard
# Authtoken: https://dashboard.ngrok.com/get-started/your-authtoken
# API key:   https://dashboard.ngrok.com/api
export NGROK_AUTHTOKEN="..."
export NGROK_API_KEY="..."








```

2) Install Gateway API CRDs (standard) and the `GatewayClass`





```bash
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.3.0/standard-install.yaml

kubectl apply -f -<<'EOF'
apiVersion: gateway.networking.k8s.io/v1
kind: GatewayClass
metadata:
  name: ngrok
spec:
  controllerName: ngrok.com/gateway-controller
EOF








```

3) Install the ngrok Operator (secure method recommended)





```bash
# Store credentials as a Secret used by the Operator
kubectl apply -f -<<'EOF'
apiVersion: v1
kind: Secret
metadata:
  name: ngrok-operator-credentials
  namespace: ngrok-operator
data:
  API_KEY: "$(echo -n "$NGROK_API_KEY" | base64)"
  AUTHTOKEN: "$(echo -n "$NGROK_AUTHTOKEN" | base64)"
EOF

helm install ngrok-operator ngrok/ngrok-operator \


  --namespace ngrok-operator \


  --create-namespace \


  --set credentials.secret.name=ngrok-operator-credentials \


  --set replicaCount=2
# Optional: --set region=us|eu|ap|jp|in|sa
# Optional: --set watchNamespace=<namespace-to-watch>








```

Validation:





```bash
kubectl -n ngrok-operator get pods
kubectl get gatewayclasses








```

### Phase 2 — Establish the Gateway on public.5dlabs.ai
Create a `Gateway` with HTTPS listener for `public.5dlabs.ai`. The operator reserves the domain with ngrok and exposes a CNAME target.





```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: Gateway
metadata:
  name: public-gateway
  namespace: default
spec:
  gatewayClassName: ngrok
  listeners:
    - name: https
      protocol: HTTPS
      port: 443
      hostname: "public.5dlabs.ai"
      allowedRoutes:
        namespaces:
          from: All








```

Apply and fetch the ngrok CNAME target:





```bash
kubectl apply -f public-gateway.yaml
kubectl -n default get gateway public-gateway -o yaml | yq '.status.addresses'


# Example:
# - type: Hostname
#   value: abc123.def456.ngrok-cname.com








```

Create DNS:
- In your DNS provider, set a **CNAME** record: `public.5dlabs.ai` → `abc123.def456.ngrok-cname.com`
- Note: CNAMEs are valid on subdomains (e.g., `public.5dlabs.ai`), not on the zone apex.

Readiness check:





```bash
kubectl get gateways.gateway.networking.k8s.io --all-namespaces
# Wait for PROGRAMMED=True on public-gateway once DNS propagates.








```

TLS:


- ngrok issues/serves certificates automatically once the CNAME resolves.

### Phase 3 — Route GitHub webhooks to Argo Events
You can route directly (Argo Events validates signature), or pre-verify using ngrok Traffic Policy.

Identify the GitHub EventSource Service:





```bash
kubectl -n argo-events get svc
# Choose the Service and port (often ~12000 for GitHub eventsource)








```

Option A — Route only (no pre-verification at ngrok):





```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: github-webhooks
  namespace: argo-events
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: default
  hostnames:


    - "public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: /github/webhook
      backendRefs:
        - kind: Service
          name: github-eventsource-svc   # replace with your Service name
          port: 12000                    # replace with your Service port








```

Option B — Pre-verify with ngrok Traffic Policy, then route:





```yaml
apiVersion: ngrok.k8s.ngrok.com/v1alpha1
kind: NgrokTrafficPolicy
metadata:
  name: verify-github-webhook
  namespace: argo-events
spec:
  policy:
    on_http_request:
      - actions:
          - type: verify-webhook
            config:
              provider: github
              secret: your-shared-secret   # set this to your GitHub webhook secret
              enforce: true


---
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: github-webhooks
  namespace: argo-events
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: default
  hostnames:


    - "public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: /github/webhook
      backendRefs:
        - kind: Service
          name: github-eventsource-svc   # replace with your Service name
          port: 12000                    # replace with your Service port
      filters:
        - type: ExtensionRef
          extensionRef:
            group: ngrok.k8s.ngrok.com
            kind: NgrokTrafficPolicy
            name: verify-github-webhook








```

Configure GitHub:
- Set the webhook URL to `https://public.5dlabs.ai/github/webhook`


- If using Option B, set the webhook secret to match `your-shared-secret`

### Phase 4 — Publish additional public apps
Add more `HTTPRoute`s under the same host (path-based), or additional listeners/hostnames (host-based):

Path-based example (same host):





```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: app1-route
  namespace: default
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: default
  hostnames:


    - "public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: /app1
      backendRefs:
        - kind: Service
          name: app1-svc
          port: 80








```

Host-based example (new subdomain):





```yaml
# Add listener to the Gateway (and later add DNS CNAME for app1.public.5dlabs.ai)
apiVersion: gateway.networking.k8s.io/v1
kind: Gateway
metadata:
  name: public-gateway
  namespace: default
spec:
  gatewayClassName: ngrok
  listeners:
    - name: https
      protocol: HTTPS
      port: 443
      hostname: "public.5dlabs.ai"
      allowedRoutes:
        namespaces:
          from: All
    - name: app1
      protocol: HTTPS
      port: 443
      hostname: "app1.public.5dlabs.ai"
      allowedRoutes:
        namespaces:
          from: All


---
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: app1-host-route
  namespace: default
spec:
  parentRefs:
    - group: gateway.networking.k8s.io
      kind: Gateway
      name: public-gateway
      namespace: default
  hostnames:


    - "app1.public.5dlabs.ai"
  rules:
    - matches:
        - path:
            type: PathPrefix
            value: /
      backendRefs:
        - kind: Service
          name: app1-svc
          port: 80








```

Then create a CNAME: `app1.public.5dlabs.ai` → the CNAME target reported in the `Gateway` status for that listener.

### Troubleshooting
- Check Gateway programming:





```bash
kubectl get gateways.gateway.networking.k8s.io --all-namespaces
kubectl -n default get gateway public-gateway -o yaml | yq '.status'








```

- If a listener is invalid (e.g., HTTP on 443), the `Accepted/Programmed` conditions explain why. Use valid `protocol`/`port` pairs: HTTP:80, HTTPS:443.


- Confirm DNS CNAME is correct and propagated.
- Check Operator logs:





```bash
kubectl -n ngrok-operator logs deploy/ngrok-operator | tail -n 200








```

### Security considerations


- Keep ngrok credentials in a Kubernetes `Secret` (as configured above).


- Webhook verification can run at ngrok via `verify-webhook` and/or at Argo Events.


- Use Traffic Policy for IP restrictions, auth, header controls as needed.

### Rollback / Uninstall




```bash
# Remove routes and gateway if needed
kubectl delete httproute -n argo-events github-webhooks --ignore-not-found
kubectl delete gateway -n default public-gateway --ignore-not-found

# Uninstall operator
helm -n ngrok-operator uninstall ngrok-operator || true
kubectl -n ngrok-operator delete secret ngrok-operator-credentials --ignore-not-found

# (Optional) Remove GatewayClass and CRDs
kubectl delete gatewayclass ngrok --ignore-not-found
# CRDs are shared; remove only if you’re sure no other controllers use them








```



### Notes for GitOps


- If deploying via Argo CD, template the above manifests/Helm config into your GitOps repo and let Argo apply changes. Ensure secrets are sourced from your secret store rather than inline values.


