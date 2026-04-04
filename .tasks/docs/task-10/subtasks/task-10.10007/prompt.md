Implement subtask 10007: Create NetworkPolicy resources restricting egress traffic

## Objective
Create Kubernetes NetworkPolicy resources to restrict traffic flow: PM server can reach bridge services, GitHub API, Hermes API, and Linear API. Frontend can only reach PM server. Block all other egress from sigma-1-dev except DNS.

## Steps
1. Create `manifests/production/network-policies.yaml`.
2. Define a default-deny egress NetworkPolicy for the sigma-1-dev namespace:
   - podSelector: {} (all pods)
   - policyTypes: [Egress]
   - egress: allow DNS (UDP port 53 to kube-dns).
3. Define a PM server egress NetworkPolicy:
   - podSelector: matchLabels app=sigma-1-pm-server
   - egress rules allowing:
     - To pods matching labels for bridge services (discord-bridge, linear-bridge) within the namespace.
     - To external IPs/CIDRs for GitHub API (140.82.112.0/20, 192.30.252.0/22), Linear API, and Hermes API. Alternatively, if external egress can't be IP-restricted, allow all egress on port 443 from PM server only.
     - DNS (UDP 53).
4. Define a frontend egress NetworkPolicy:
   - podSelector: matchLabels app=sigma-1-frontend
   - egress: allow to PM server pods only (podSelector matchLabels app=sigma-1-pm-server) on the service port, plus DNS.
5. Apply: `kubectl apply -f manifests/production/network-policies.yaml`.
6. Test connectivity from pods.

## Validation
`kubectl get networkpolicy -n sigma-1-dev` returns at least 3 NetworkPolicy resources. `kubectl exec` into PM server pod: `curl -s discord-bridge-http:port/health` succeeds. `kubectl exec` into PM server pod: `curl -s --connect-timeout 3 http://example.com` times out or is refused (blocked by default deny). `kubectl exec` into frontend pod: `curl -s sigma-1-pm-server:8080/health` succeeds.