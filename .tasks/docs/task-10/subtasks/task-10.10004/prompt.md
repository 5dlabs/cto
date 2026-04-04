Implement subtask 10004: Harden Cilium NetworkPolicies with deny-all default and exact port allowlisting

## Objective
Create an explicit deny-all default CiliumNetworkPolicy for the sigma-1-dev namespace, then author allowlist policies with exact port numbers for all required ingress and egress paths.

## Steps
1. Create a default deny-all CiliumNetworkPolicy for the namespace:
   ```yaml
   apiVersion: cilium.io/v2
   kind: CiliumNetworkPolicy
   metadata:
     name: default-deny-all
     namespace: sigma-1-dev
   spec:
     endpointSelector: {}
     ingress:
     - {}
     egress:
     - {}
   ```
   Note: An empty ingress/egress with `endpointSelector: {}` denies all. Use the correct Cilium deny-all pattern: `ingressDeny` and `egressDeny` or simply omit ingress/egress rules to deny.
2. Create allowlist policies for PM server ingress: allow TCP port 3000 from the ingress controller pods only.
3. Create allowlist policies for PM server egress:
   - To CloudNative-PG: TCP port 5432, pod selector `cnpg.io/cluster=sigma-1-pg`.
   - To Redis: TCP port 6379 (and 26379 for Sentinel), pod selector for Redis pods.
   - To bridge service: exact port, pod selector.
   - To external APIs (Linear, GitHub, Hermes): use `toFQDNs` or `toCIDRSet` with exact ports (443).
   - To DNS (kube-dns): UDP port 53 to `kube-system` namespace.
4. Create allowlist policies for CloudNative-PG replication traffic between PG pods: TCP 5432.
5. Create allowlist policies for Redis Sentinel inter-node traffic: TCP 6379 and 26379.
6. Apply all policies and verify no existing connectivity is broken for legitimate traffic.
7. Test that unauthorized connections are blocked.

## Validation
Deploy a test pod in sigma-1-dev with no matching allowlist: `kubectl run nettest --image=busybox -n sigma-1-dev -- sleep 3600`. Attempt `wget -qO- --timeout=5 http://sigma-1-pm-server:3000` from that pod and assert it times out or is refused. Then verify PM server can still reach Postgres: exec into PM pod and confirm DB connectivity. Verify PM server can reach Redis: exec and run `redis-cli ping`. Verify PM server egress to external APIs works by curling a known endpoint.