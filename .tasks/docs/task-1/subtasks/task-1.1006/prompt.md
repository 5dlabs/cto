Implement subtask 1006: Implement basic internal network policies

## Objective
Apply basic network policies within the 'sigma1' namespace to allow internal service communication, ensuring services can reach PostgreSQL and Redis/Valkey.

## Steps
1. Define a NetworkPolicy resource that allows ingress traffic from pods within the 'sigma1' namespace to other pods in the same namespace.2. Define a NetworkPolicy that allows egress traffic from 'sigma1' pods to the 'databases' namespace for PostgreSQL and Redis/Valkey access.3. Apply these NetworkPolicies.

## Validation
1. Verify NetworkPolicies are applied using `kubectl get networkpolicy -n sigma1`.2. Deploy a test pod in 'sigma1' and attempt to connect to 'sigma1-postgres' and 'sigma1-valkey' to confirm connectivity.