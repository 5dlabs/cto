Implement subtask 10004: Network policies: default deny all ingress/egress in sigma1 namespace

## Objective
Create a default-deny NetworkPolicy in the sigma1 namespace that blocks all ingress and egress traffic by default.

## Steps
Step-by-step:
1. Create `netpol-default-deny.yaml`:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: default-deny-all
     namespace: sigma1
   spec:
     podSelector: {}
     policyTypes:
       - Ingress
       - Egress
   ```
2. This policy selects all pods in sigma1 and denies all traffic. Subsequent allow policies will open specific paths.
3. WARNING: Apply this AFTER the allow policies are ready, or apply them all atomically. Otherwise all sigma1 services will lose connectivity.
4. Add a namespace label `networking/default-deny: 'true'` for documentation purposes.

## Validation
Apply the deny policy in isolation on a test namespace first. Spin up a test pod and verify it cannot reach any external endpoint or other pods. Then verify that adding a specific allow policy restores that specific path only.