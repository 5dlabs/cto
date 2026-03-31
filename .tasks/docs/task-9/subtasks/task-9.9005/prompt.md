Implement subtask 9005: Define and apply Kubernetes network policies for inter-service traffic restriction

## Objective
Create NetworkPolicy resources to enforce least-privilege network access between services. Only explicitly allowed traffic should be permitted; all other inter-pod communication should be denied by default.

## Steps
1. Create a default-deny-all ingress and egress NetworkPolicy in the application namespace:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: default-deny-all
   spec:
     podSelector: {}
     policyTypes: [Ingress, Egress]
   ```
2. Create allow-ingress policies for each service based on actual communication patterns:
   - Web frontend → API service (allow on API port)
   - API service → PostgreSQL (allow on 5432)
   - API service → Redis (allow on 6379)
   - Morgan → API service (allow on API port)
   - cloudflared → web and Morgan services (allow on service ports)
3. Create allow-egress policies:
   - All pods → kube-dns (UDP/TCP 53)
   - Application pods → external APIs (if needed, restrict to specific CIDRs or use FQDN if CNI supports it)
   - PostgreSQL replicas ↔ primary (allow on 5432 for replication)
   - Redis replicas ↔ Sentinel (allow on 6379 and 26379)
4. Label all pods consistently to support selector-based policies.
5. Verify the CNI plugin supports NetworkPolicy (e.g., Calico, Cilium).

## Validation
After applying policies: verify allowed traffic works (e.g., API can reach PostgreSQL on 5432). Verify denied traffic is blocked (e.g., web frontend cannot directly reach PostgreSQL). Use `kubectl exec` from a test pod to attempt connections that should be blocked and confirm they time out. Verify DNS resolution still works from all pods.