Implement subtask 1008: Apply Cilium NetworkPolicy CRDs for PM server traffic paths

## Objective
Create and apply Cilium NetworkPolicy (CiliumNetworkPolicy) CRDs in the `sigma-1-dev` namespace that allow only declared traffic paths from the PM server to Postgres, Redis, discord-bridge-http, and linear-bridge, plus egress to external Linear/GitHub/Hermes APIs. Deny all other intra-namespace traffic.

## Steps
1. Create a default-deny CiliumNetworkPolicy for the `sigma-1-dev` namespace: deny all ingress and egress for pods with no matching policy.
2. Create a CiliumNetworkPolicy allowing PM server pods (label selector `app=sigma-1-pm`) egress to `sigma-1-pg` pods on port 5432 (TCP).
3. Create a CiliumNetworkPolicy allowing PM server pods egress to `sigma-1-redis` pods on port 6379 (TCP).
4. Create a CiliumNetworkPolicy allowing PM server pods egress to `discord-bridge-http` service in the `bots` namespace on its service port.
5. Create a CiliumNetworkPolicy allowing PM server pods egress to `linear-bridge` service in the `bots` namespace on its service port.
6. Create a CiliumNetworkPolicy allowing PM server pods egress to external endpoints: Linear API (api.linear.app:443), GitHub API (api.github.com:443), and Hermes API (specify FQDN or CIDR). Use Cilium FQDN-based egress rules where possible.
7. Allow DNS egress (port 53 UDP/TCP to kube-dns) for all pods in the namespace.
8. Apply all policy manifests.
9. Verify the expected number of CiliumNetworkPolicy resources exist in the namespace.

## Validation
`kubectl get cnp -n sigma-1-dev --no-headers | wc -l` returns the expected count (approximately 7 policies). `kubectl get cnp -n sigma-1-dev -o jsonpath='{.items[*].metadata.name}'` lists all expected policy names. Cilium agent reports no policy enforcement errors: `kubectl -n kube-system exec ds/cilium -- cilium policy get` shows imported policies.