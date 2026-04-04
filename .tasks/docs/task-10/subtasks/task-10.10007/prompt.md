Implement subtask 10007: Network policies: allow Morgan (openclaw) to sigma1 services and ingress controller to sigma1

## Objective
Create NetworkPolicy allowing Morgan pods in openclaw namespace to reach sigma1 service ports, and allowing the ingress controller to reach sigma1 public endpoints.

## Steps
Step-by-step:
1. **Morgan → sigma1 ingress policy** in sigma1 namespace:
   - `podSelector: {}` or specific public service labels
   - `ingress[0].from[0].namespaceSelector.matchLabels: {name: openclaw}`, `ingress[0].from[0].podSelector.matchLabels: {app: morgan}`
   - `ingress[0].ports`: list all sigma1 service ports (e.g., 8080, 3000, etc. for each service Morgan calls)
2. **Ingress controller → sigma1 ingress policy**:
   - `ingress[0].from[0].namespaceSelector.matchLabels: {name: ingress-system}` (or whatever namespace the ingress controller/cloudflared runs in)
   - `ingress[0].ports`: list public service ports
3. Verify the ingress controller namespace label exists; add it if missing.

## Validation
From a pod with Morgan labels in openclaw namespace, curl each sigma1 service endpoint and verify 200 responses. From the ingress controller namespace, verify traffic flows to sigma1 services. From an unlabeled pod in a random namespace, verify access is denied.