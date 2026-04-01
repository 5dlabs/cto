Implement subtask 10007: Create NetworkPolicy restricting inter-pod and egress communication

## Objective
Define and apply NetworkPolicy resources that restrict frontend pods to only reach PM server on port 3000, and restrict PM server egress to external APIs while blocking cross-namespace traffic.

## Steps
1. Create `infra/network-policies/frontend-netpol.yaml`:
   - podSelector matching frontend pods (e.g., label `app: frontend`).
   - Ingress: deny all (frontend is accessed via Ingress controller, add separate rule if needed).
   - Egress: allow traffic only to PM server pods on port 3000 (podSelector `app: pm-server`, port 3000). Deny all other egress.
2. Create `infra/network-policies/pm-server-netpol.yaml`:
   - podSelector matching PM server pods (label `app: pm-server`).
   - Ingress: allow from frontend pods on port 3000. Allow from Ingress controller if PM server is directly exposed.
   - Egress: allow DNS (port 53 UDP/TCP to kube-dns). Allow traffic to external IPs/CIDRs for Linear API, GitHub API, Discord webhooks, Nous API (use `0.0.0.0/0` with namespace restriction, or specific CIDRs if known). Deny egress to other namespaces using namespaceSelector deny rules.
3. Create `infra/network-policies/default-deny.yaml`:
   - Default deny all ingress and egress for the `sigma1-prod` namespace as a baseline.
4. Apply in order: default-deny first, then specific allow policies.
5. Verify: `kubectl get networkpolicies -n sigma1-prod`.

## Validation
From a frontend pod, run `curl pm-server:3000/health` — should succeed. From a frontend pod, run `curl https://api.github.com` — should time out or be refused. From a PM server pod, run `curl https://api.github.com` — should succeed. From a PM server pod, attempt to reach a service in another namespace — should fail.