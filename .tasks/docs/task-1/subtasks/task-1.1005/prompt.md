Implement subtask 1005: Apply network policies for egress to Linear, Discord, GitHub, and Nous APIs

## Objective
Create and apply Kubernetes NetworkPolicy resources in sigma1-dev allowing egress traffic to Linear API, Discord webhooks, GitHub API, and Nous/Hermes API endpoints while denying all other egress by default.

## Steps
1. Create a default-deny egress NetworkPolicy `netpol-default-deny-egress.yaml` targeting all pods in sigma1-dev.
2. Create an allow-egress NetworkPolicy `netpol-allow-external-apis.yaml` that permits egress to:
   - Linear API: resolve `api.linear.app` IPs or use CIDR blocks; port 443.
   - Discord webhooks: resolve `discord.com` IPs or use CIDR blocks; port 443.
   - GitHub API: resolve `api.github.com` IPs or use CIDR blocks; port 443.
   - Nous/Hermes API: resolve the Hermes endpoint IPs; port 443.
3. Also allow egress to kube-dns (UDP/TCP 53) on the cluster DNS CIDR so pods can resolve hostnames.
4. Also allow intra-namespace traffic for pod-to-pod communication (PM server ↔ auxiliary services).
5. Apply all NetworkPolicy manifests: `kubectl apply -f netpol-*.yaml -n sigma1-dev`.

## Validation
`kubectl get networkpolicy -n sigma1-dev` lists the default-deny and allow policies. `kubectl describe networkpolicy -n sigma1-dev` shows correct egress rules for ports 443 and 53. Policy audit confirms no unexpected egress CIDRs are allowed.