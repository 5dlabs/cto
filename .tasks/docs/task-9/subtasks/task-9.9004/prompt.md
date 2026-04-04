Implement subtask 9004: Create default-deny NetworkPolicy for sigma-1 namespace

## Objective
Apply a default-deny-all NetworkPolicy for both ingress and egress traffic in the sigma-1 namespace, establishing a zero-trust baseline.

## Steps
Step-by-step:
1. Create a default-deny NetworkPolicy:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: default-deny-all
     namespace: sigma-1
     labels:
       sigma-1-pipeline: production
   spec:
     podSelector: {}
     policyTypes:
     - Ingress
     - Egress
   ```
2. Apply: `kubectl apply -f default-deny-all.yaml`.
3. IMPORTANT: This will immediately block ALL traffic to/from pods in sigma-1. The allow-rule subtasks (9005, 9006) must be applied in the same deployment window or before this policy.
4. Verify the policy exists: `kubectl get networkpolicy -n sigma-1`.

## Validation
`kubectl get networkpolicy default-deny-all -n sigma-1` exists. A test pod launched in sigma-1 without matching allow rules cannot reach any endpoint (connection timeout). DNS resolution from pods should also be blocked (this confirms egress deny is active).